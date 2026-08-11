# Light and Shadow — The Sun and Moon Over the Land

The sky clock ([26](26-living-terrain.md) §4) gains a position, and the land gains relief. Pure presentation, derived entirely from authoritative state (the sim hour, the heightmap, the tree scatter, the moon and tilt in `tuning::Sky`): the ledgers never see a shadow, and the shadows never lie about the ledgers.

## 1. Where the light is

The light source has an **azimuth and an altitude** derived from the hour and the year: rising east, arcing overhead, setting west; noon altitude follows the season and latitude band (a winter sun stays low — its shadows stay long all day; under high axial tilt the polar summer sun circles without setting). At night the **moon takes over** as a dimmer source with the same geometry, its strength following the lunar phase — a full-moon night has visible relief, a new-moon night is shape-less dark. One shared model feeds every scale.

## 2. Relief on the world map

The heightmap gets **hillshading against the live sun**: slopes facing the light warm and brighten, slopes facing away fall into shade, computed from the elevation gradient and the current sun vector. Because the vector moves with the clock, the map's relief *turns* through the day — east faces lit at dawn, west faces at evening — and lengthening shade at low sun makes mountains read as mountains. The shading layer rebuilds when the displayed hour changes and composes under the seasonal terrain tint and over the ground colors ([27](27-the-ground.md)), so a snowy ridge at winter dawn looks like exactly that.

## 3. Cast shadows at person scale

On local maps, **trees and standing things cast real shadows**: an offset dark shape per tree, direction opposite the sun, length from its altitude — long at dawn, pooled at noon, long again at dusk, and swinging as the hours pass. Moonlight casts the same shadows faintly. People walk through them; the felled tree's shadow vanishes with it. (Works and buildings join the caster list as they gain visible bodies.)

## 4. Later slices

True occlusion shadows on the world map — ridges actually darkening the valleys behind them at low sun, via a horizon sweep along the light direction (still per-frame cheap, still no renderer physics); clouds as moving shade when weather events land ([26] W1); firelight at camps as a night source at person scale; and the render-core ([10](10-visualization.md)) inheriting all of it when the pixel-art era arrives.
