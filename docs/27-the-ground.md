# The Ground — Regolith as Composition, Never a Rock Enum

"What is the soil made of here?" gets a real answer. Every land tile carries a **regolith**: a composition across a grain-size ladder — bedrock, coarse (scree and gravel), sand, fines (silt and clay), and organic matter — plus nothing else. There is no `SoilType` enum and never will be: "gravel scree", "shifting sand", "living loam", and "desert" are *descriptions of regions in composition space*, exactly as species are regions of trait space ([19](19-ecology-and-subsistence.md)) and blood is a region of substance space ([23](23-bodies-and-substances.md)). Manipulation, not naming, is the point: every process that touches the ground — weathering, water, wind, roots, and later picks and plows — is a **movement through the space**.

## 1. The ladder and its movements

Composition per tile sums to a constant: the ground is always made of *something*, so every loss is an exposure and every gain is a burial. The movements, all monthly integer field passes ([26](26-living-terrain.md) discipline, all rates in `tuning::Ground`):

- **Frost-shattering**: tiles that bank snow and then thaw ([26]'s snowpack knows exactly where) break rock toward coarse and coarse toward sand — mountains grind themselves into scree because winters actually happen there.
- **Heat-cracking**: hot, dry, bare ground weathers rock toward sand slowly.
- **Wash**: delivered water (rain + melt, straight from the climate pass) carries fines down `drains_to`, more where the drop is steep. The donor's surface coarsens toward what lies beneath — stripped hillsides turn stony — and the receiver is buried in fines: **floodplains and deltas become the soft, rich ground because that is where the silt goes**.
- **Roots and rot**: living vegetation builds organic matter; bare, hot ground loses it. The green makes its own soil, and losing the green loses the soil after it.
- **Winnowing**: dry, bare tiles have their fines blown out, shifting toward sand.

**Desertification is a sentence-length emergent loop**: dry ground loses plants → loses organic and fines → holds less water → grows less → loses more. A desert is not placed; it is *arrived at*, and it can spread — or be arrested by whatever keeps the green alive. The reverse loop builds loam under stable forest. Nobody authors either.

## 2. Fertility becomes a consequence

`cell_fertility` stops being a frozen genesis verdict. Live fertility is **derived from the composition** — organic and fines rich, sand and rock poor — so cultivation yields now sit on top of the whole causal chain: mountains shed silt, rivers deliver it, plants bind it, and the plow spends it. Cultivation's own soil cost (depletion under sustained farming, restoration under fallow) is the E-series soil slot landing on real material. Genesis keeps a static fertility as the *habitability baseline* (a niche judgment, like climate for species); the living number drives the yields and the reports.

## 3. Extraction is an interface, not a feature (yet)

The composition **is** the future mining and building surface: bedrock fraction and hardness are what quarries and mines will bid against; coarse is aggregate; fines are the potter's clay and the brickmaker's mud. When the [invention grammar](03a-grammar-spec.md) lands, material inputs for designs read from this layer — the ground was already made of manipulable stuff, so nothing needs retrofitting. Until then the layer simply *is*, colors the world, and moves.

## 4. Seen and said

The terrain's base color now comes from its composition — sand country reads pale gold, silt beds dark, scree grey, loam deep — so **regions look different because they are made of different things, and change color as they change substance** (watch a floodplain darken over decades; watch an overgrazed steppe blanch). The tile inspector answers the question directly, in one legible line (docs/21 duty): "mostly sand, some fines, thin organic — shifting country" versus "deep fines under heavy organic — river loam".

**Ships now (G0):** the regolith state and genesis derivation from existing fields (slope, water, flora, climate); frost/heat weathering, wash along `drains_to`, roots/rot, winnowing; derived live fertility wired into cultivation and the inspector; composition-colored terrain; describe lines. **Designed, queued:** cultivation depletion/fallow; elevation actually lowering under sustained erosion (the [26] W1 slot); extraction verbs when the grammar or works need them; dunes and loess as wind-borne deposition (winnowing's other half); landslides on oversteep scree.
