# Ecology and Subsistence — The Living World, and Who Eats What

E0 of the [economy program](18-economy.md), built the only way it can honestly be built: food comes from an **ecosystem**, and how a people feeds itself is an **emergent outcome of comparative returns** — never a named stage of history. This document is the deep design; the v1 implementation ships its core loop.

## 1. The kingdoms coexist

Three living layers over the same tile fields ([13](13-worldgen.md)), each with its own dynamics, all coupled:

- **Flora** (exists, now *dynamic*): per-tile vegetation density stops being a genesis constant. It regrows logistically toward the settled baseline, and is drawn down by grazing, gathering, and (later) logging and clearing. Deforestation and overgrazing become real, visible states of the map.
- **Fauna** (new): per-world generated animal genomes in the flora tradition — climate tolerances plus **continuous trait axes, not authored taxa**: `diet` (plant-eater ↔ flesh-eater) and `water` (terrestrial ↔ aquatic) are spectra, reproduction speed falls out of diet, and "grazer"/"predator"/"fish" are *descriptions* of trait space — omnivores and semi-aquatic hunters emerge. Flora likewise: growth form is a `woodiness` spectrum (grass → old forest), not a Grass/Shrub/Tree enum. Species counts per kingdom live in tuning. Populations live per tile, grow and starve by Lotka-Volterra-flavored fixed-point dynamics, and diffuse toward better habitat. Prey crashes starve predators; predator loss lets grazers strip the range. Biomes emerge as *distinct food webs*: what a steppe, a riverland, and a forest can feed is different because different things live there.
- **Fungi/decomposers** are deferred, but the layer slot exists (nutrient cycling would close the loop; noted, not built).

The [invention grammar's](03a-grammar-spec.md) stratified-generation + competition method is reused for fauna; a fourth species-generation pass later ties fauna to flora species (specialist grazers) rather than density alone. Beneath the trait axes, every species now carries a generated **body plan** — organs, limbs, senses, and a working fluid — from the anatomy grammar ([23](23-bodies-and-substances.md)); edibility and (later) wounds derive from it.

## 2. Subsistence channels, not subsistence identities

A nation allocates its labor across **extraction channels** — that allocation *is* its way of life, and nobody names it:

| Channel | Draws on | Character |
|---|---|---|
| **Gather** | edible share of flora density | Low setup, modest steady yield, depletes and recovers with the plants |
| **Hunt** | grazer + predator biomass | High yield while game is rich, high variance, depletes fast, recovers slowly |
| **Fish** | aquatic fauna in river/lake/coast tiles | Steady where water is; nothing where it isn't |
| **Herd** | captured grazers as living stock | Converts pasture to food; must be *seeded from wild grazers*; herds eat the range and scale with it |
| **Cultivate** | soil fertility × water | Starts near-worthless; **establishment** builds over months of sustained labor into the highest yield-per-land anywhere fertile — a slow expensive bet, not a default |

Rules that make the emergence honest:

- **No channel is gated by "era" or doctrine.** All five exist from tick one; conditions and knowledge (later, grammar techs like plows and nets shift the coefficients) decide what pays.
- **Sedentism and nomadism are outcomes.** Depleting returns present two exits: *intensify* (cultivation establishment, herd building — investments that anchor you) or *move* (the autopilot relocates a starving band to richer ground, abandoning the tile). Bands that keep moving are nomads; bands that invested are settled; nobody flagged them as either.
- **Mixed economies are normal.** Real allocations are portfolios (fish + gather + a growing herd); the report shows the portfolio, not a label.

## 3. Demographics eat from the ledger

The crowding-based capacity formula **is deleted**. Cohort drives now carry **nutrition** (food consumed ÷ food needed): births scale down and deaths scale up as nutrition falls; famine is an empty larder (`nutrition < 0.75`), not a density proxy. Granaries become real storage caps; farmstead works multiply cultivation yield; stores spoil monthly. Population curves are henceforth explained entirely by the food ledger — E0's exit criterion.

## 4. The agent-bias problem, answered structurally

LLM overseers arrive with Earth's script: "civilizations become agrarian." Four mechanisms keep the sim from ratifying that prior:

1. **No teleological vocabulary anywhere agents read.** Reports and charters never say hunter-gatherer, agrarian, nomad, advanced, primitive, era, or progress. They say: *this tile, this month — gather 0.9/worker, hunt 1.6 falling, fish 1.2 steady, cultivate 0.3 rising with establishment 22%, herd requires wild grazers (present).* Decisions are made against numbers, not narratives.
2. **Honest comparative economics.** Each channel's coefficients are tuned so its real-world *logic* holds without its real-world *frequency* being assumed: cultivation is the best yield-per-land **only** on fertile watered land and only after months of sunk labor; herding beats farming on marginal grass; fishing beats both where water is rich; hunting is unbeatable at low population density. Earth's answer emerges on Earth-like tiles; other tiles get other answers.
3. **The ungoverned baseline is a return-follower.** Autopilot labor allocation greedily follows marginal yields (with a small inertia term). NPC bands therefore diversify or specialize by *landscape*, providing an unbiased control group visible in every world — if agent-led nations all rush cultivation on bad land while NPCs herd past them, the bias is measurable and the failure is the agent's, priced in food.
4. **Directed is allowed, priced, and consequential.** setting `labor.*` leaves costs mandate ([16](16-mandate-and-works.md)) and pins them against the autopilot; an overseer *may* force the plow against the numbers, and the ledger will grade the choice. Preconception isn't forbidden — it's billed.

## 5. v1 scope vs. deferred

**Ships now:** dynamic flora density; ~12 fauna species (grazers, predators, aquatic) with monthly trophic dynamics, diffusion, and hunting/capture coupling; the five channels with per-nation labor weights (`labor.*` policy leaves, autopilot greedy default); nutrition-driven demographics; storage/spoilage; band relocation (emergent mobility); reports rebuilt around the food table; famine = hunger.

**Deferred, slots ready:** per-tile labor (E1 occupations); domestication traits and breeds; fauna on local maps and presence hunting scenes (P1); disease and nutrient cycling; seasonal yields (weather); fishing-stock depletion; grammar techs modifying channel coefficients; culture — a *learned* preference term in autopilot allocation, so traditions can form and be wrong.
