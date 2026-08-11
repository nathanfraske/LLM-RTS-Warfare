//! The self-describing governance surface (docs/20-open-directives.md):
//! sim systems register the levers (policy leaves) and undertakings
//! (actions) a council can use; overseers steer through generic
//! `Set`/`Enact` directives validated against this registry. Adding a
//! lever is a registration, never a schema change.

mod tree;

pub use tree::{PolicyLeaf, PolicyTree};

use serde::{Deserialize, Serialize};

/// A value a policy leaf or action parameter can hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PolicyValue {
    Int(i64),
    Text(String),
}

impl PolicyValue {
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Text(_) => None,
        }
    }

    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Int(_) => None,
            Self::Text(v) => Some(v),
        }
    }
}

impl std::fmt::Display for PolicyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Text(v) => write!(f, "{v}"),
        }
    }
}

/// What values a leaf or parameter admits; bounds are enforced server-side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// An integer in `min..=max`.
    IntRange { min: i64, max: i64 },
    /// One of a closed set of named options.
    Choice { options: Vec<String> },
    /// Free text, trimmed, 1..=`max_len` characters.
    Text { max_len: usize },
}

impl PolicyType {
    /// Reject values outside what this type admits.
    pub fn check(&self, value: &PolicyValue) -> Result<(), String> {
        match (self, value) {
            (Self::IntRange { min, max }, PolicyValue::Int(v)) => {
                if v < min || v > max {
                    Err(format!("value {v} is outside {min}..={max}"))
                } else {
                    Ok(())
                }
            }
            (Self::Choice { options }, PolicyValue::Text(v)) => {
                if options.iter().any(|o| o == v) {
                    Ok(())
                } else {
                    Err(format!("\"{v}\" is not one of: {}", options.join(", ")))
                }
            }
            (Self::Text { max_len }, PolicyValue::Text(v)) => {
                let trimmed = v.trim();
                if trimmed.is_empty() || trimmed.len() > *max_len {
                    Err(format!("text must be 1..={max_len} characters"))
                } else {
                    Ok(())
                }
            }
            _ => Err("wrong value shape for this entry".into()),
        }
    }

    /// The bounds, as agents read them in a report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::IntRange { min, max } => format!("{min}..{max}"),
            Self::Choice { options } => options.join(" / "),
            Self::Text { max_len } => format!("text, up to {max_len} chars"),
        }
    }
}

/// One registered lever: a per-nation behavior knob a sim system reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyDef {
    /// Dotted path, e.g. `labor.hunt`.
    pub key: String,
    pub kind: PolicyType,
    pub default: PolicyValue,
    /// Mandate cost of setting it, before autonomy scaling.
    pub cost: f64,
    /// One agent-facing line: numbers and mechanisms, never eras.
    pub summary: String,
}

/// What an action must be aimed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    /// No target tile: the action concerns the nation itself.
    Nation,
    /// A tile the nation owns.
    OwnedTile,
    /// An unowned land tile bordering the nation's territory.
    FrontierTile,
}

/// One named parameter an action takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    pub kind: PolicyType,
}

/// One registered undertaking a council can enact at a target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDef {
    /// Dotted path, e.g. `works.commission`.
    pub key: String,
    pub target: TargetKind,
    pub params: Vec<ParamDef>,
    /// Mandate cost, before autonomy scaling.
    pub cost: f64,
    pub summary: String,
}

/// Every lever and action alive in this world — assembled at genesis from
/// each system's registration, rendered into every report.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Registry {
    pub policies: Vec<PolicyDef>,
    pub actions: Vec<ActionDef>,
}

impl Registry {
    #[must_use]
    pub fn policy(&self, key: &str) -> Option<&PolicyDef> {
        self.policies.iter().find(|d| d.key == key)
    }

    #[must_use]
    pub fn action(&self, key: &str) -> Option<&ActionDef> {
        self.actions.iter().find(|d| d.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_admit_and_reject() {
        let range = PolicyType::IntRange { min: 0, max: 1000 };
        assert!(range.check(&PolicyValue::Int(350)).is_ok());
        assert!(range.check(&PolicyValue::Int(1350)).is_err());
        assert!(range.check(&PolicyValue::Text("350".into())).is_err());

        let choice = PolicyType::Choice {
            options: vec!["steady".into(), "expansive".into()],
        };
        assert!(choice.check(&PolicyValue::Text("steady".into())).is_ok());
        assert!(choice.check(&PolicyValue::Text("bold".into())).is_err());

        let text = PolicyType::Text { max_len: 8 };
        assert!(text.check(&PolicyValue::Text("Ash".into())).is_ok());
        assert!(text.check(&PolicyValue::Text("   ".into())).is_err());
        assert!(
            text.check(&PolicyValue::Text("far too long".into()))
                .is_err()
        );
    }
}
