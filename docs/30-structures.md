# Structures — Built from What the Ground Gives, Named by What They Are

The works catalog was the last little authored list wearing a registry costume: three entries, no materials, no integrity beyond "the newest falls". This document replaces it with **structures as composition**: a building is a *function* realized in *materials bid from the local ground*, its name, cost, effects, and strength all derived — the [grammar](03a-grammar-spec.md) move, applied to shelter, a step ahead of the grammar itself.

## 1. Aspects are the authored floor; buildings are not

The first cut of this design authored three *functions* (field-works, store-house, hearth-hall) — still a list of what buildings may do, as Nathan pointed out. The floor now sits a level down, where it belongs: what we author is the vocabulary of **physical aspects** — the ways a built thing can couple to state the sim already has: `cover` (kept off the sky), `enclosure` (held apart from the world), `capacity` (room to hold things), `worked ground` (land made ready), `hearth` (heat held against the cold). These are closed over current state the way subsistence channels are closed over food sources, and they open by registration when new state arrives (walls against foes when war lands; height for seeing when watchtowers meet the knowledge layer). **Nothing in the sim dispatches on a building's name or kind**: every effect reads aspect numbers, and nouns like "long-store" and "field-works" are describe-words over the space — regions, exactly as "omnivore" and "loam" are.

A commission names an **effort emphasis** — roomy, sheltering, ground-working, hearth-warm, balanced — an allocation over the aspect dimensions, not a building type. The building that answers derives from **what the tile actually offers**: earth from the fines, stone from the scree and the bedrock's own hardness ([29](29-the-underground.md)), timber from the standing green, thatch and turf from the light growth. Wall, roof, and footing each go to the best material present; the name is read off the composition. Two nations spending the same effort build different buildings because they stand on different ground — **regional architecture emerges from geology**, unauthored.

## 1a. The people build unbidden — if the council allows

Buildings are no longer only commissioned. When stores overflow their room, when families crowd an uncovered tile, when established fields want working, **the people raise what they need themselves** — no decree, no mandate, one raising per nation per month, the design derived from their own ground like any other. And whether they *may* is itself governance: the policy leaf `building.initiative` ("unbidden" / "council-only") sits in the registry beside the expansion posture, so a council can claim the sole right to build — and live with a people who wait on its word. Test-pinned both ways: twelve fat years move somebody to build; forbidden people build nothing.

## 2. Everything a building is, derived

- **Cost**: build months derive from mass and hardness — stone is slow, mud is quick.
- **Effect**: from composition and function — a stone-walled store keeps more against vermin and weather than a mud one; a tight-bound warm-roofed hall shelters better. The economy keeps its interfaces (cultivation multiplier, storage capacity, birth shelter) but the *numbers* come from the walls.
- **Integrity**: from materials against their job — binding and hardness carry, span loads, and the **footing meets the actual regolith**: build on rock and stand; build on sand and lean ([27](27-the-ground.md) finally prices construction ground). Integrity is spent, not toggled: quakes ([29]) deal damage scaled against it, heavy ash loads crush weak roofs, fire ([26]) scorches what can burn — a structure survives what its materials can survive, collapses when its integrity is spent, and the chronicle names what fell by what it was.

## 3. What this is not yet, honestly

True invention ("figuring out how to") arrives with the grammar: designs as *discovered* compositions rather than emphasis presets, new aspects as registered couplings, materials bid against mineral properties for tools and machines. The five emphases are effort presets, not discoveries — the people build what their situation asks from what their ground gives, but they do not yet invent forms nobody has seen. Also queued: structures on the local map rendered from their real materials; ruins as regolith deposits when things fall; repair and upkeep labor; works as targets in war.
