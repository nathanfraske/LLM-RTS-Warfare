//! The monthly trophic tick: logistic growth against living carrying
//! capacities, predation, grazing pressure on the flora, and diffusion
//! toward better habitat. Fixed-point, deterministic, order-fixed.

use crate::{Fauna, Trophic, carrying};
use tuning::Ecology;
use world_map::WorldFields;
use world_schema::Quantity;

/// One month of wild dynamics. Mutates populations and the living flora.
pub fn tick_month(fauna: &mut Fauna, fields: &WorldFields, flora_live: &mut [u8], eco: &Ecology) {
    let cells = fauna.cells();
    grow(fauna, fields, flora_live, cells, eco);
    graze(fauna, flora_live, fields, cells, eco);
    diffuse(fauna, fields, flora_live, cells, eco);
}

/// Logistic growth; predators grow against prey and eat them.
fn grow(fauna: &mut Fauna, fields: &WorldFields, flora_live: &[u8], cells: usize, eco: &Ecology) {
    for si in 0..fauna.species.len() {
        let s = fauna.species[si].clone();
        let r = Quantity::from_num(s.repro_milli) / Quantity::from_num(1000);
        for (t, &fl) in flora_live.iter().enumerate().take(cells) {
            let p = fauna.at(si, t);
            if p <= Quantity::ZERO {
                continue;
            }
            let k = match s.trophic {
                Trophic::Predator => {
                    let prey = prey_biomass(fauna, t);
                    (prey * Quantity::from_num(eco.predator_k_prey_frac))
                        .min(carrying(&s, fields, fl, t, eco))
                }
                _ => carrying(&s, fields, fl, t, eco),
            };
            let next = if k <= Quantity::from_num(1) {
                // Habitat gone: die back hard.
                p * Quantity::from_num(eco.collapse_keep)
            } else {
                let crowd = (Quantity::ONE - p / k).max(Quantity::from_num(-1));
                (p + p * r * crowd).max(Quantity::ZERO)
            };
            fauna.set(si, t, next);
            if s.trophic == Trophic::Predator {
                eat_prey(
                    fauna,
                    t,
                    next * Quantity::from_num(eco.predator_demand_frac),
                    eco,
                );
            }
        }
    }
}

fn prey_biomass(fauna: &Fauna, tile: usize) -> Quantity {
    fauna
        .species
        .iter()
        .filter(|x| x.trophic == Trophic::Grazer)
        .fold(Quantity::ZERO, |acc, x| acc + fauna.at(x.id as usize, tile))
}

/// Predation debits grazers proportionally, sparing a refuge fraction.
fn eat_prey(fauna: &mut Fauna, tile: usize, demand: Quantity, eco: &Ecology) {
    let total = prey_biomass(fauna, tile);
    if total <= Quantity::ZERO {
        return;
    }
    let eaten = demand.min(total * Quantity::from_num(eco.predation_max_frac));
    for si in 0..fauna.species.len() {
        if fauna.species[si].trophic != Trophic::Grazer {
            continue;
        }
        let p = fauna.at(si, tile);
        fauna.set(si, tile, p - eaten * (p / total));
    }
}

/// Heavy grazer load wears the vegetation down toward a floor.
fn graze(fauna: &Fauna, flora_live: &mut [u8], fields: &WorldFields, cells: usize, eco: &Ecology) {
    for (t, fl) in flora_live.iter_mut().enumerate().take(cells) {
        if fields.elevation[t] < 0 {
            continue;
        }
        let load = prey_biomass(fauna, t);
        let k_full = Quantity::from_num(eco.grazer_k_full) * Quantity::from_num(*fl)
            / Quantity::from_num(255);
        if k_full <= Quantity::ZERO {
            continue;
        }
        let ratio = (load / k_full).min(Quantity::from_num(2));
        let wear = (ratio * Quantity::from_num(eco.grazing_pressure)).to_num::<i64>();
        *fl = fl
            .saturating_sub(u8::try_from(wear.clamp(0, 8)).expect("clamped"))
            .max(eco.flora_floor);
    }
}

/// A share of overcrowded stocks flows to the fittest neighbor.
fn diffuse(
    fauna: &mut Fauna,
    fields: &WorldFields,
    flora_live: &[u8],
    cells: usize,
    eco: &Ecology,
) {
    for si in 0..fauna.species.len() {
        let s = fauna.species[si].clone();
        for t in 0..cells {
            let p = fauna.at(si, t);
            let here = carrying(&s, fields, flora_live[t], t, eco);
            if p <= here / Quantity::from_num(2) || p <= Quantity::from_num(2) {
                continue;
            }
            let (neighbors, n) = fields.grid().neighbors8(t);
            let best = neighbors[..n].iter().copied().max_by_key(|&nb| {
                (
                    carrying(&s, fields, flora_live[nb], nb, eco).to_bits(),
                    usize::MAX - nb,
                )
            });
            if let Some(nb) = best
                && carrying(&s, fields, flora_live[nb], nb, eco) > here
            {
                let moving = p * Quantity::from_num(eco.diffusion_frac);
                fauna.set(si, t, p - moving);
                let there = fauna.at(si, nb);
                fauna.set(si, nb, there + moving);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::WorldSeed;

    fn world() -> (WorldFields, Vec<u8>, Fauna) {
        let fields = WorldFields::generate(WorldSeed(42), 64);
        let flora = flora::settle::settle(WorldSeed(42), &fields, 24);
        let fauna = Fauna::genesis(WorldSeed(42), &fields, &flora.density, &Ecology::default());
        (fields, flora.density, fauna)
    }

    #[test]
    fn wildlife_persists_and_is_deterministic() {
        let (fields, mut flora_a, mut a) = world();
        let (_, mut flora_b, mut b) = world();
        let eco = Ecology::default();
        for _ in 0..24 {
            tick_month(&mut a, &fields, &mut flora_a, &eco);
            tick_month(&mut b, &fields, &mut flora_b, &eco);
        }
        assert_eq!(a.pop, b.pop);
        assert_eq!(flora_a, flora_b);
        let total: Quantity = a.pop.iter().fold(Quantity::ZERO, |x, y| x + *y);
        assert!(
            total > Quantity::from_num(1000),
            "the wild must not go extinct"
        );
    }

    #[test]
    fn predators_starve_without_prey() {
        let (fields, mut flora, mut fauna) = world();
        // Exterminate all grazers everywhere.
        for si in 0..fauna.species.len() {
            if fauna.species[si].trophic == Trophic::Grazer {
                for t in 0..fauna.cells() {
                    fauna.set(si, t, Quantity::ZERO);
                }
            }
        }
        let before: Quantity = predator_total(&fauna);
        let eco = Ecology::default();
        for _ in 0..18 {
            tick_month(&mut fauna, &fields, &mut flora, &eco);
        }
        let after = predator_total(&fauna);
        assert!(
            after < before / Quantity::from_num(4),
            "predators must crash without prey: {before:.0} -> {after:.0}"
        );
    }

    fn predator_total(fauna: &Fauna) -> Quantity {
        fauna
            .species
            .iter()
            .filter(|s| s.trophic == Trophic::Predator)
            .fold(Quantity::ZERO, |acc, s| {
                (0..fauna.cells()).fold(acc, |a, t| a + fauna.at(s.id as usize, t))
            })
    }
}
