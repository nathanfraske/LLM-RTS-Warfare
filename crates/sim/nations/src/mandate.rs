//! Mandate: the people's readiness to be commanded, and the autonomy
//! friction that compounds under direct rule (docs/16-mandate-and-works.md).
//! Base prices live on registry entries; every other number is in
//! `tuning::Society`.

use tuning::Society;
use world_schema::Quantity;

/// Effective cost of a base-priced intervention after autonomy friction.
#[must_use]
pub fn effective_cost(base: Quantity, autonomy: Quantity, soc: &Society) -> Quantity {
    base * (Quantity::ONE + autonomy / Quantity::from_num(soc.autonomy_cost_divisor))
}

/// Spend for one paid intervention: deduct mandate, raise autonomy.
pub fn spend(mandate: &mut Quantity, autonomy: &mut Quantity, cost: Quantity, soc: &Society) {
    *mandate -= cost;
    if cost > Quantity::ZERO {
        *autonomy = (*autonomy + Quantity::from_num(soc.autonomy_per_spend))
            .min(Quantity::from_num(soc.autonomy_cap));
    }
}

/// Monthly upkeep: regen (slowed by autonomy) and autonomy decay.
pub fn tick_month(mandate: &mut Quantity, autonomy: &mut Quantity, soc: &Society) {
    let regen = Quantity::from_num(soc.mandate_regen)
        * (Quantity::ONE - *autonomy / Quantity::from_num(soc.autonomy_regen_divisor));
    *mandate = (*mandate + regen).min(Quantity::from_num(soc.mandate_cap));
    *autonomy *= Quantity::from_num(soc.autonomy_decay_keep);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autonomy_makes_direct_rule_harder_and_restraint_heals_it() {
        let soc = Society::default();
        let base = Quantity::from_num(soc.cost_stance);
        let mut mandate = Quantity::from_num(soc.starting_mandate);
        let mut autonomy = Quantity::ZERO;
        let first = effective_cost(base, autonomy, &soc);
        spend(&mut mandate, &mut autonomy, first, &soc);
        let second = effective_cost(base, autonomy, &soc);
        spend(&mut mandate, &mut autonomy, second, &soc);
        let later = effective_cost(base, autonomy, &soc);
        assert!(later > first, "costs must escalate under micromanagement");

        let before_regen = mandate;
        for _ in 0..6 {
            tick_month(&mut mandate, &mut autonomy, &soc);
        }
        assert!(mandate > before_regen, "restraint restores mandate");
        assert!(
            effective_cost(base, autonomy, &soc) < later,
            "autonomy decays with restraint"
        );
        assert!(mandate <= Quantity::from_num(soc.mandate_cap));
    }

    #[test]
    fn free_interventions_leave_no_friction() {
        let soc = Society::default();
        let mut mandate = Quantity::from_num(soc.starting_mandate);
        let mut autonomy = Quantity::ZERO;
        spend(&mut mandate, &mut autonomy, Quantity::ZERO, &soc);
        assert_eq!(autonomy, Quantity::ZERO);
        assert_eq!(mandate, Quantity::from_num(soc.starting_mandate));
    }
}
