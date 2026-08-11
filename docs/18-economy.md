# The Economy Program

The economy is not a milestone; it is a program — the deep system most other depth hangs from. This is the bigger plan: principles, the goods model, five phases with exit criteria, and what each phase makes real elsewhere. Nothing here ships as a shallow slice; each phase replaces a placeholder with a mechanism.

## Principles

1. **Goods are physical and conserved.** Every good is a ledgered quantity with sources, sinks, and exact fixed-point accounting — the same discipline as population. No "+10% economy" abstractions anywhere.
2. **Demand comes from needs.** Cohorts consume by need schedules (food always; warmth, tools, comforts as they exist). Unmet needs have consequences (famine, unrest, emigration) — demand is never a curve, it's people.
3. **Labor is time.** A cohort's month is an allocation across occupations. Institutions allocate within policy weights ([04](04-institutions-directives.md)); everything produced costs somebody's hours.
4. **Prices emerge locally.** No global market object. Exchange ratios derive from local scarcity, and divergence between places is what makes trade — and trade routes — worth fighting over.
5. **Fields-first inputs.** Yields derive from the tile fields and flora that already exist ([13](13-worldgen.md)): fertility feeds farms, forest density feeds logging, geology (future field) feeds mining.
6. **Scale by layout.** Per-tile SoA good vectors, monthly batch dynamics, GPU-ready — the market-clearing kernel was always on the M4 list ([01](01-architecture.md)).

## The goods model

A schema-first **goods registry** (data, not code — [01 §6](01-architecture.md)): id, category, spoilage rate, bulk (logistics weight), and need-satisfaction mappings. v1 basket: `food`, `wood`, `stone`, `hides`; tools and refined goods arrive with production chains and, eventually, from the [invention grammar](03a-grammar-spec.md) (a design's outputs are goods; the grammar's cost model finally lands on real ledgers).

Per-tile state: `stock[good]`, monthly `produced[good]` / `consumed[good]`, storage caps (granaries become real containers). All `Quantity`, all conserved, all evented.

## Phases

**E0 — shipped, deeper than planned** ([19-ecology-and-subsistence](19-ecology-and-subsistence.md)): food comes from a living ecology through five subsistence channels rather than a yield formula; the notes below stand as the original sketch.

**E0 — Stocks and yields.** Tiles produce food/wood/stone monthly from fields × labor share; cohorts eat; surplus stores (with spoilage); shortfall starves. **Famine stops being a crowding proxy and becomes an empty larder** — capacity as a formula retires, replaced by what the land actually feeds. Works get real outputs (farmstead = food yield, granary = storage cap). *Exit: a nation's population curve is explained entirely by its food ledger; the crowding-based capacity function is deleted.*

**E1 — Labor and occupations.** The cohort key gains occupation ([02](02-simulation-core.md) always planned it). Institutions allocate labor monthly within policy weights; construction consumes labor + materials, so commissions ([16](16-mandate-and-works.md)) get real costs and build times derived from workforce. *Exit: reallocating labor via policy visibly shifts output; a work stalls without hands or timber.*

**E2 — Chains and buildings.** Production recipes over building modules ([07](07-buildings-and-cities.md) begins): wood→charcoal, hides→leather, tool-multipliers on yields. Storage, spoilage, and stockpile management become decisions. *Exit: a two-step chain is strictly better than raw extraction and visibly operates on local maps.*

**E3 — Exchange.** Internal redistribution between a nation's tiles first; then inter-nation trade along adjacency and routes: barter ratios from relative scarcity, caravans that carry actual goods ([15](15-multiscale-maps.md) world layer), trade events in the feed. Trade gives [06-diplomacy](06-diplomacy-intel.md) its stakes — embargoes, tribute ([12](12-sovereignty.md)) enforceable in goods, and something worth a war. *Exit: cutting a trade route measurably hurts both parties; agents can see it in reports and react.*

**E4 — Logistics.** Supply for armies ([09](09-battles.md) stops hand-waving "supply state"), route capacity, seasonal yield variation (the weather hook, [13](13-worldgen.md)). *Exit: an army outrunning its grain starves before it fights.*

## What each phase feeds

Presence tasks ([17](17-presence.md) P2) earn against E0 budgets — chopping is real the month E0 lands. Reports grow ledgers (E0), labor tables (E1), and trade sheets (E3) — the overseer's decision surface deepens with each phase. Mandate costs stay the *consent* price while goods become the *material* price of works. And tuning ([14](14-bands-and-councils.md) rates, [16](16-mandate-and-works.md) numbers) happens per-phase against real mechanisms, not once against proxies.

## Order of battle

E0 is the recommended next implementation target after this document: it deletes the biggest proxy (capacity), gives presence something real to animate, and every later phase composes on its ledgers.
