//! What the nations system registers on the governance surface: its policy
//! leaves and its actions (docs/20-open-directives.md). Registration is the
//! whole contract — reports, validation, and pricing all read the registry.

use policy::{ActionDef, ParamDef, PolicyDef, PolicyType, PolicyValue, TargetKind};
use tuning::Society;

/// Expansion posture leaf: read by the band autopilot's split logic.
pub const POSTURE: &str = "expansion.posture";
pub const POSTURE_CONSOLIDATE: &str = "consolidate";
pub const POSTURE_STEADY: &str = "steady";
pub const POSTURE_EXPANSIVE: &str = "expansive";

/// Whether the people may raise buildings on their own need, unbidden —
/// a lever, because whether they may is the council's to decide.
pub const BUILDING_INITIATIVE: &str = "building.initiative";
pub const INITIATIVE_UNBIDDEN: &str = "unbidden";
pub const INITIATIVE_COUNCIL_ONLY: &str = "council-only";

pub const NAME: &str = "nation.name";
pub const SETTLE: &str = "band.settle";
pub const COMMISSION: &str = "works.commission";
pub const SCOUT: &str = "band.scout";

#[must_use]
pub fn policy_defs(soc: &Society) -> Vec<PolicyDef> {
    vec![
        PolicyDef {
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
        },
        PolicyDef {
            key: BUILDING_INITIATIVE.into(),
            kind: PolicyType::Choice {
                options: vec![INITIATIVE_UNBIDDEN.into(), INITIATIVE_COUNCIL_ONLY.into()],
            },
            default: PolicyValue::Text(INITIATIVE_UNBIDDEN.into()),
            cost: soc.cost_stance,
            summary: "Whether the people may raise buildings on their own need, or only by \
                      council commission."
                .into(),
        },
    ]
}

#[must_use]
pub fn action_defs(soc: &Society) -> Vec<ActionDef> {
    let emphasis_options = structures::EMPHASES
        .iter()
        .map(|e| (*e).to_string())
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
            summary: "Decree settlement of a bordering tile your people have walked; a band \
                      founds it when it can."
                .into(),
        },
        ActionDef {
            key: SCOUT.into(),
            target: TargetKind::Nation,
            params: vec![ParamDef {
                name: "bearing".into(),
                kind: PolicyType::Choice {
                    options: knowledge::BEARING_NAMES
                        .iter()
                        .map(|b| (*b).to_string())
                        .collect(),
                },
            }],
            cost: soc.cost_scout,
            summary: "Send a party out from the seat to walk a bearing and map what it crosses; \
                      the map updates when they return — if they return."
                .into(),
        },
        ActionDef {
            key: COMMISSION.into(),
            target: TargetKind::OwnedTile,
            params: vec![ParamDef {
                name: "emphasis".into(),
                kind: PolicyType::Choice {
                    options: emphasis_options,
                },
            }],
            cost: soc.cost_commission,
            summary: "Raise a structure by effort emphasis — what gets built derives from \
                      the tile's own ground; its room, cover, worked ground, and hearth do \
                      the rest."
                .into(),
        },
    ]
}
