# Roadmap

| Milestone | Scope | Status |
|---|---|---|
| **M1 Drawing foundation** | canvas, vector strokes, pressure, layers, timeline, PNG/SVG | **done** |
| **M2 AI drawing brain** | stroke model interface, GRU inference, stroke-5 dataset I/O | interface done, **no weights** |
| **M3 Anime intelligence** | proportions, eyes, hair, expression, character consistency | **done for bust portraits** |
| **M4 Autonomous artist** | plan, draw, self-evaluate, correct | **done, geometric only** |
| **M4b Real physics / maths foundations** | sRGB/Oklab linear-light colour, analytic Bézier arc length, XPBD constraint solver, Lambert/GGX lighting — as an independent zero-dependency crate | **implemented; colour + arc length wired, lighting/XPBD not yet** |
| **M5 Manga studio** | panels, poses, colouring, animation | **not started** |

The M4b crate (`crates/buncraft-science/`) is real, tested maths. Two of its
four areas are now used by the drawing pipeline: `Rgba::mix_linear` for
additive highlights, and exact arc-length sampling for every spline stroke.
Lighting (`light::shade`) and the XPBD hair solver are still unwired, and the
rasteriser still composites in sRGB because `tiny-skia` does its own blending.
See `research/physics_math_science/science_plan.md` for the remaining sequence.

## Next concrete steps

0. **Finish wiring the science crate (incremental).** Colour and arc length are
   done. Remaining: drive `stage_shading()` from `light::shade` (2.5D hemisphere
   normal), then optional XPBD hair. The rasteriser's sRGB compositing is
   blocked on replacing `tiny-skia`'s internal blend, not on maths. Do not add
   `nalgebra`/`kurbo`/`rapier`/`avian` as direct dependencies until the MSRV
   1.80 vs 1.84–1.89 conflict is resolved (`palette` is compatible but is not
   used, to keep the science crate independent).
1. **Pose model.** Add `Pose` to `CharacterProfile` (head yaw/pitch, shoulder
   tilt) and drive `Geometry` from it. Unlocks 3/4 and profile views.
2. **Stroke dataset bridge.** Export BunCraft drawings as stroke-5 and train the
   GRU on them as a self-supervised baseline; measure reconstruction error, not
   aesthetics.
3. **Path booleans.** Integrate a clipping library so hair can cut the face
   contour and lineart intersections stay clean.
4. **Animation.** Keyframes over action parameters, linear in-betweens, frame
   export to MP4 via an encoder crate.
5. **Plotter export.** Douglas–Peucker simplification, HPGL and G-code writers.

## Non-goals

- Photorealistic rendering.
- Diffusion/image-generation wrappers.
- Copying GPL or unlicensed code from the studied repositories.
