//! The report's charter: the live governance surface, rendered from the
//! registry — every lever and action with current values, bounds, and
//! prices, never a hand-written list (docs/20-open-directives.md).

use std::fmt::Write as _;

use nations::Nation;
use policy::{Registry, TargetKind};
use tuning::Tuning;
use world_schema::{Quantity, Tick};

pub(crate) fn charter(
    out: &mut String,
    nation: &Nation,
    registry: &Registry,
    now: Tick,
    tun: &Tuning,
) {
    let mult =
        Quantity::ONE + nation.autonomy / Quantity::from_num(tun.society.autonomy_cost_divisor);
    let _ = writeln!(out, "\n## Council directives\n");
    let _ = writeln!(
        out,
        "Steer by appending JSON to the directive log; entries apply at their \
         tick (current tick {}). Costs are mandate, ×{mult:.2} at current \
         autonomy. Two forms:\n",
        now.0
    );
    let _ = writeln!(out, "```json");
    let _ = writeln!(
        out,
        "{{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Set\", \
         \"key\": \"<lever>\", \"value\": <admitted value> }} }}",
        now.0, nation.id.0
    );
    let _ = writeln!(
        out,
        "{{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Enact\", \
         \"action\": \"<action>\", \"target\": <tile id>, \"params\": {{ \"<param>\": <value> }} }} }}",
        now.0, nation.id.0
    );
    let _ = writeln!(out, "```");
    levers(out, nation, registry);
    actions(out, registry);
}

fn levers(out: &mut String, nation: &Nation, registry: &Registry) {
    let _ = writeln!(
        out,
        "\nLevers (Set) — (decreed) marks a value pinned by council order; \
         the rest follow returns:\n"
    );
    let _ = writeln!(out, "| Lever | Current | Admits | Cost | Effect |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for def in &registry.policies {
        let current = nation
            .policy
            .value(&def.key)
            .map_or_else(|| "—".into(), ToString::to_string);
        let mark = if nation.policy.directed(&def.key) {
            " (decreed)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {} | {current}{mark} | {} | {} | {} |",
            def.key,
            def.kind.describe(),
            def.cost,
            def.summary
        );
    }
}

fn actions(out: &mut String, registry: &Registry) {
    let _ = writeln!(out, "\nActions (Enact):\n");
    let _ = writeln!(out, "| Action | Target | Params | Cost | Effect |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for def in &registry.actions {
        let target = match def.target {
            TargetKind::Nation => "—",
            TargetKind::OwnedTile => "an owned tile id",
            TargetKind::FrontierTile => "a bordering unowned tile id",
        };
        let params = if def.params.is_empty() {
            "—".to_string()
        } else {
            def.params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.kind.describe()))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        let _ = writeln!(
            out,
            "| {} | {target} | {params} | {} | {} |",
            def.key, def.cost, def.summary
        );
    }
}
