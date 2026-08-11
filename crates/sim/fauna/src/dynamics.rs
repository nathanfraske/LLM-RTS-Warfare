//! The monthly trophic tick, trait-driven: every genome grows against the
//! food its diet actually reaches — plants by its plant share, prey by its
//! flesh share — predation pressure distributes over plant-leaning bodies on
//! the matching side of the waterline, grazing wears the vegetation, and
//! overcrowded stocks diffuse toward better habitat. Fixed-point,
//! deterministic, order-fixed.

use crate::{Fauna, carrying};
use tuning::Ecology;
use world_map::WorldFields;
use world_schema::Quantity;

/// One month of wild dynamics. Mutates populations and the living flora.
pub fn tick_month(fauna: &mut Fauna, fields: &WorldFields, flora_live: &mut [u8], eco: &Ecology) {
    let cells = fauna.cells();
    grow_and_predate(fauna, fields, flora_live, cells, eco);
    graze(fauna, flora_live, fields, cells, eco);
    diffuse(fauna, fields, flora_live, cells, eco);
}

/// Plant-leaning biomass per side of the waterline — what flesh-eaters reach.
fn prey_pools(fauna: &Fauna, tile: usize) -> (Quantity, Quantity) {
    let mut land = Quantity::ZERO;
    let mut water = Quantity::ZERO;
    for s in &fauna.species {
        let p = fauna.at(s.id as usize, tile) * s.plant_frac();
        land += p * s.land_frac();
        water += p * s.water_frac();
    }
    (land, water)
}

fn grow_and_predate(
    fauna: &mut Fauna,
    fields: &WorldFields,
    flora_live: &[u8],
    cells: usize,
    eco: &Ecology,
) {
    for (t, &fl) in flora_live.iter().enumerate().take(cells) {
        let (prey_land, prey_water) = prey_pools(fauna, t);
        let mut demand_land = Quantity::ZERO;
        let mut demand_water = Quantity::ZERO;

        for si in 0..fauna.species.len() {
            let s = fauna.species[si].clone();
            let p = fauna.at(si, t);
            if p <= Quantity::ZERO {
                continue;
            }
            let r = Quantity::from_num(s.repro_milli) / Quantity::from_num(1000);
            let habitat = carrying(&s, fields, fl, t, eco);
            // Flesh share of the diet feeds on prey pools, not the land itself.
            let own_prey = p * s.plant_frac();
            let reachable_prey = (prey_land * s.land_frac() + prey_water * s.water_frac()
                - own_prey)
                .max(Quantity::ZERO);
            let k = habitat * s.plant_frac()
                + (reachable_prey * Quantity::from_num(eco.predator_k_prey_frac)).min(habitat)
                    * s.flesh_frac();
            let next = if k <= Quantity::from_num(1) {
                p * Quantity::from_num(eco.collapse_keep)
            } else {
                let crowd = (Quantity::ONE - p / k).max(Quantity::from_num(-1));
                (p + p * r * crowd).max(Quantity::ZERO)
            };
            fauna.set(si, t, next);
            let demand = next * s.flesh_frac() * Quantity::from_num(eco.predator_demand_frac);
            demand_land += demand * s.land_frac();
            demand_water += demand * s.water_frac();
        }

        // Predation lands proportionally on plant-leaning stocks per side.
        let eaten_land = demand_land.min(prey_land * Quantity::from_num(eco.predation_max_frac));
        let eaten_water = demand_water.min(prey_water * Quantity::from_num(eco.predation_max_frac));
        if eaten_land > Quantity::ZERO || eaten_water > Quantity::ZERO {
            for si in 0..fauna.species.len() {
                let s = fauna.species[si].clone();
                let p = fauna.at(si, t);
                if p <= Quantity::ZERO {
                    continue;
                }
                let contrib = p * s.plant_frac();
                let mut loss = Quantity::ZERO;
                if prey_land > Quantity::ZERO {
                    loss += eaten_land * (contrib * s.land_frac() / prey_land);
                }
                if prey_water > Quantity::ZERO {
                    loss += eaten_water * (contrib * s.water_frac() / prey_water);
                }
                fauna.set(si, t, p - loss);
            }
        }
    }
}

/// Heavy plant-eating land biomass wears the vegetation toward a floor.
fn graze(fauna: &Fauna, flora_live: &mut [u8], fields: &WorldFields, cells: usize, eco: &Ecology) {
    for (t, fl) in flora_live.iter_mut().enumerate().take(cells) {
        if fields.elevation[t] < 0 {
            continue;
        }
        let (load, _) = prey_pools(fauna, t);
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
        let fauna = Fauna::genesis(
            WorldSeed(42),
            &fields,
            &flora.density,
            &Ecology::default(),
            &tuning::Bodies::default(),
        );
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
    fn flesh_eaters_starve_when_plant_eaters_vanish() {
        let (fields, mut flora, mut fauna) = world();
        // Exterminate every plant-leaning genome everywhere.
        for si in 0..fauna.species.len() {
            if fauna.species[si].diet_milli < 450 {
                for t in 0..fauna.cells() {
                    fauna.set(si, t, Quantity::ZERO);
                }
            }
        }
        let before = flesh_total(&fauna);
        let eco = Ecology::default();
        for _ in 0..18 {
            tick_month(&mut fauna, &fields, &mut flora, &eco);
        }
        let after = flesh_total(&fauna);
        assert!(
            after < before / Quantity::from_num(3),
            "flesh-eaters must crash without prey: {before:.0} -> {after:.0}"
        );
    }

    fn flesh_total(fauna: &Fauna) -> Quantity {
        fauna
            .species
            .iter()
            .filter(|s| s.diet_milli > 650)
            .fold(Quantity::ZERO, |acc, s| {
                (0..fauna.cells()).fold(acc, |a, t| a + fauna.at(s.id as usize, t))
            })
    }
}
