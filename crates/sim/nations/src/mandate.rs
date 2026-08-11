//! Mandate: the people's readiness to be commanded, and the autonomy
//! friction that compounds under direct rule (docs/16-mandate-and-works.md).
//! Every number lives in `tuning::Society`.

use directive_schema::Directive;
use tuning::Society;
use world_schema::Quantity;

/// Base mandate cost of a directive; autonomy scales it up.
#[must_use]
pub fn base_cost(directive: &Directive, soc: &Society) -> Quantity {
    match directive {
        Directive::Name { .. } => Quantity::ZERO,
        Directive::SetStance { .. } => Quantity::from_num(soc.cost_stance),
        Directive::SetLabor { .. } => Quantity::from_num(soc.cost_labor),
        Directive::Settle { .. } => Quantity::from_num(soc.cost_settle),
        Directive::Commission { .. } => Quantity::from_num(soc.cost_commission),
    }
}

/// Effective cost after autonomy friction.
#[must_use]
pub fn effective_cost(directive: &Directive, autonomy: Quantity, soc: &Society) -> Quantity {
    base_cost(directive, soc)
        * (Quantity::ONE + autonomy / Quantity::from_num(soc.autonomy_cost_divisor))
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
    use directive_schema::Stance;

    #[test]
    fn autonomy_makes_direct_rule_harder_and_restraint_heals_it() {
        let soc = Society::default();
        let stance = Directive::SetStance {
            stance: Stance::Expansive,
        };
        let mut mandate = Quantity::from_num(soc.starting_mandate);
        let mut autonomy = Quantity::ZERO;
        let first = effective_cost(&stance, autonomy, &soc);
        spend(&mut mandate, &mut autonomy, first, &soc);
        let second = effective_cost(&stance, autonomy, &soc);
        spend(&mut mandate, &mut autonomy, second, &soc);
        let later = effective_cost(&stance, autonomy, &soc);
        assert!(later > first, "costs must escalate under micromanagement");

        let before_regen = mandate;
        for _ in 0..6 {
            tick_month(&mut mandate, &mut autonomy, &soc);
        }
        assert!(mandate > before_regen, "restraint restores mandate");
        assert!(
            effective_cost(&stance, autonomy, &soc) < later,
            "autonomy decays with restraint"
        );
        assert!(mandate <= Quantity::from_num(soc.mandate_cap));
    }

    #[test]
    fn naming_is_free() {
        assert_eq!(
            base_cost(&Directive::Name { name: "x".into() }, &Society::default()),
            Quantity::ZERO
        );
    }
}
