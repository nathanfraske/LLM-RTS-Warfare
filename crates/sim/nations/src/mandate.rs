//! Mandate: the people's readiness to be commanded, and the autonomy
//! friction that compounds under direct rule (docs/16-mandate-and-works.md).

use directive_schema::Directive;
use world_schema::Quantity;

pub const MANDATE_CAP: f64 = 10.0;
pub const STARTING_MANDATE: f64 = 6.0;
const REGEN_PER_MONTH: f64 = 1.2;
const AUTONOMY_PER_SPEND: f64 = 6.0;
const AUTONOMY_CAP: f64 = 100.0;

/// Base mandate cost of a directive; autonomy scales it up.
#[must_use]
pub fn base_cost(directive: &Directive) -> Quantity {
    match directive {
        Directive::Name { .. } => Quantity::ZERO,
        Directive::SetStance { .. } => Quantity::from_num(1),
        Directive::Settle { .. } => Quantity::from_num(2),
        Directive::Commission { .. } => Quantity::from_num(3),
    }
}

/// Effective cost after autonomy friction: base × (1 + autonomy/60).
#[must_use]
pub fn effective_cost(directive: &Directive, autonomy: Quantity) -> Quantity {
    base_cost(directive) * (Quantity::ONE + autonomy / Quantity::from_num(60))
}

/// Spend for one paid intervention: deduct mandate, raise autonomy.
pub fn spend(mandate: &mut Quantity, autonomy: &mut Quantity, cost: Quantity) {
    *mandate -= cost;
    if cost > Quantity::ZERO {
        *autonomy = (*autonomy + Quantity::from_num(AUTONOMY_PER_SPEND))
            .min(Quantity::from_num(AUTONOMY_CAP));
    }
}

/// Monthly upkeep: regen (slowed by autonomy) and autonomy decay.
pub fn tick_month(mandate: &mut Quantity, autonomy: &mut Quantity) {
    let regen =
        Quantity::from_num(REGEN_PER_MONTH) * (Quantity::ONE - *autonomy / Quantity::from_num(200));
    *mandate = (*mandate + regen).min(Quantity::from_num(MANDATE_CAP));
    *autonomy *= Quantity::from_num(0.95);
}

#[cfg(test)]
mod tests {
    use super::*;
    use directive_schema::Stance;

    #[test]
    fn autonomy_makes_direct_rule_harder_and_restraint_heals_it() {
        let stance = Directive::SetStance {
            stance: Stance::Expansive,
        };
        let mut mandate = Quantity::from_num(STARTING_MANDATE);
        let mut autonomy = Quantity::ZERO;
        let first = effective_cost(&stance, autonomy);
        spend(&mut mandate, &mut autonomy, first);
        let second = effective_cost(&stance, autonomy);
        spend(&mut mandate, &mut autonomy, second);
        let later = effective_cost(&stance, autonomy);
        assert!(later > first, "costs must escalate under micromanagement");

        let before_regen = mandate;
        for _ in 0..6 {
            tick_month(&mut mandate, &mut autonomy);
        }
        assert!(mandate > before_regen, "restraint restores mandate");
        assert!(
            effective_cost(&stance, autonomy) < later,
            "autonomy decays with restraint"
        );
        assert!(mandate <= Quantity::from_num(MANDATE_CAP));
    }

    #[test]
    fn naming_is_free() {
        assert_eq!(
            base_cost(&Directive::Name { name: "x".into() }),
            Quantity::ZERO
        );
    }
}
