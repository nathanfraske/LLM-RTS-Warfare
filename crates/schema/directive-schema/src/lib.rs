//! The two generic directives — the only way an overseer steers a nation
//! (docs/20-open-directives.md). What may be set or enacted is not listed
//! here: the registry a world assembles at genesis decides, and every
//! report renders it. This schema never grows another verb.

use std::collections::BTreeMap;

use policy::PolicyValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Directive {
    /// Set a registered policy leaf; the leaf stays pinned against the
    /// autopilot afterwards. Costs the leaf's registered mandate price.
    Set { key: String, value: PolicyValue },
    /// Enact a registered action, aimed at a target tile where the action
    /// demands one. Costs the action's registered mandate price.
    Enact {
        action: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<u32>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        params: BTreeMap<String, PolicyValue>,
    },
}

/// One logged council decision: apply `directive` to `nation` at `tick`.
/// A JSON array of these is the overseer input stream (replay input).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectiveEntry {
    pub tick: u64,
    pub nation: u32,
    pub directive: Directive,
}
