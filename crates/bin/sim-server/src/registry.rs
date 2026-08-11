//! Assembling the world's governance surface: the composition root gathers
//! every system's registered levers and actions into one registry
//! (docs/20-open-directives.md). New systems join by adding a line here.

use policy::Registry;
use tuning::Society;

#[must_use]
pub fn assemble(soc: &Society) -> Registry {
    Registry {
        policies: [
            nations::registry::policy_defs(soc),
            economy::policy_defs(soc),
        ]
        .concat(),
        actions: nations::registry::action_defs(soc),
    }
}
