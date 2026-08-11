//! What the nations system registers on the governance surface: its policy
//! leaves and its actions (docs/20-open-directives.md). Registration is the
//! whole contract — reports, validation, and pricing all read the registry.

use policy::{ActionDef, ParamDef, PolicyDef, PolicyType, PolicyValue, TargetKind};
use tuning::Society;

use crate::works;

/// Expansion posture leaf: read by the band autopilot's split logic.
pub const POSTURE: &str = "expansion.posture";
pub const POSTURE_CONSOLIDATE: &str = "consolidate";
pub const POSTURE_STEADY: &str = "steady";
pub const POSTURE_EXPANSIVE: &str = "expansive";

pub const NAME: &str = "nation.name";
pub const SETTLE: &str = "band.settle";
pub const COMMISSION: &str = "works.commission";

#[must_use]
pub fn policy_defs(soc: &Society) -> Vec<PolicyDef> {
    vec![PolicyDef {
        key: POSTURE.into(),
        kind: PolicyType::Choice {
            options: vec![
                POSTURE_CONSOLIDATE.into(),
                POSTURE_STEADY.into(),
                POSTURE_EXPANSIVE.into(),
            ],
        },
        default: PolicyValue::Text(POSTURE_STEADY.into()),
        cost: soc.cost_stance,
        summary: "How readily crowded settlements send founders to new tiles.".into(),
    }]
}

#[must_use]
pub fn action_defs(soc: &Society) -> Vec<ActionDef> {
    let work_options = works::catalog(soc)
        .iter()
        .map(|(key, _)| (*key).to_string())
        .collect();
    vec![
        ActionDef {
            key: NAME.into(),
            target: TargetKind::Nation,
            params: vec![ParamDef {
                name: "name".into(),
                kind: PolicyType::Text { max_len: 64 },
            }],
            cost: 0.0,
            summary: "Christen the nation; the name flows into reports and the chronicle.".into(),
        },
        ActionDef {
            key: SETTLE.into(),
            target: TargetKind::FrontierTile,
            params: Vec::new(),
            cost: soc.cost_settle,
            summary: "Decree settlement of a bordering tile; a band founds it when it can.".into(),
        },
        ActionDef {
            key: COMMISSION.into(),
            target: TargetKind::OwnedTile,
            params: vec![ParamDef {
                name: "work".into(),
                kind: PolicyType::Choice {
                    options: work_options,
                },
            }],
            cost: soc.cost_commission,
            summary: "Raise a work over months — farmstead: richer fields; granary: deeper \
                      stores; dwellings: more births."
                .into(),
        },
    ]
}
