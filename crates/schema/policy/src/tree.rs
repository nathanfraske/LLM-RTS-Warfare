//! A nation's live policy state: every registered leaf with its current
//! value, and whether a council decree pinned it (the autopilot must then
//! leave it alone).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{PolicyValue, Registry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyLeaf {
    pub value: PolicyValue,
    /// True once a council decree set it; autopilot writes are refused.
    pub directed: bool,
}

/// The per-nation policy tree, keyed by registered leaf path.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyTree {
    leaves: BTreeMap<String, PolicyLeaf>,
}

impl PolicyTree {
    /// A fresh tree holding every registered leaf at its default.
    #[must_use]
    pub fn from_defaults(registry: &Registry) -> Self {
        Self {
            leaves: registry
                .policies
                .iter()
                .map(|d| {
                    (
                        d.key.clone(),
                        PolicyLeaf {
                            value: d.default.clone(),
                            directed: false,
                        },
                    )
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn value(&self, key: &str) -> Option<&PolicyValue> {
        self.leaves.get(key).map(|l| &l.value)
    }

    /// Integer leaf value, or 0 when absent — weight reads come through here.
    #[must_use]
    pub fn int(&self, key: &str) -> i64 {
        match self.value(key) {
            Some(PolicyValue::Int(v)) => *v,
            _ => 0,
        }
    }

    /// Text leaf value, or "" when absent.
    #[must_use]
    pub fn text(&self, key: &str) -> &str {
        match self.value(key) {
            Some(PolicyValue::Text(v)) => v,
            _ => "",
        }
    }

    #[must_use]
    pub fn directed(&self, key: &str) -> bool {
        self.leaves.get(key).is_some_and(|l| l.directed)
    }

    /// A council decree: set the leaf and pin it against the autopilot.
    pub fn set_directed(&mut self, key: &str, value: PolicyValue) {
        if let Some(leaf) = self.leaves.get_mut(key) {
            leaf.value = value;
            leaf.directed = true;
        }
    }

    /// An autopilot adjustment: lands only on leaves no decree has pinned.
    pub fn set_auto(&mut self, key: &str, value: PolicyValue) {
        if let Some(leaf) = self.leaves.get_mut(key)
            && !leaf.directed
        {
            leaf.value = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyDef, PolicyType};

    #[test]
    fn decrees_pin_leaves_against_the_autopilot() {
        let registry = Registry {
            policies: vec![PolicyDef {
                key: "labor.hunt".into(),
                kind: PolicyType::IntRange { min: 0, max: 1000 },
                default: PolicyValue::Int(350),
                cost: 1.0,
                summary: String::new(),
            }],
            actions: Vec::new(),
        };
        let mut tree = PolicyTree::from_defaults(&registry);
        assert_eq!(tree.int("labor.hunt"), 350);
        tree.set_auto("labor.hunt", PolicyValue::Int(400));
        assert_eq!(
            tree.int("labor.hunt"),
            400,
            "autopilot may drift free leaves"
        );
        tree.set_directed("labor.hunt", PolicyValue::Int(600));
        tree.set_auto("labor.hunt", PolicyValue::Int(100));
        assert_eq!(tree.int("labor.hunt"), 600, "decrees pin the leaf");
        assert!(tree.directed("labor.hunt"));
        assert_eq!(tree.int("labor.unknown"), 0, "absent leaves read as zero");
    }
}
