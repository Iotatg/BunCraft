# Drawing engine analysis

## Approaches observed

| Engine | Primitive | Strengths | Costs |
|---|---|---|---|
| DrawingBotV3 | polyline paths + pen settings | mature path optimisation, plotter output | GPL-3.0; Java; image-driven, not generative |
| plottter | generator → paths → hatch/stipple | rich hatching, TSP single-line, SVG/HPGL | no declared license; Python desktop |
| vector_stroke_engine | canvas 2D strokes | contour tracing + hatching in browser, SVG export | no declared license; photo-driven |
| doodlewithAI | pencil stamp with wobble/pressure/overshoot | genuinely hand-made look, replay | JS scenes, no vector export |
| whiteboard-video-engine | traced strokes + gesture | MP4 pipeline, path ordering | video-first, not editable art |
| **BunCraft** | pressure-annotated polyline + brush stamp | deterministic, vector + raster, replay, layer model | no boolean path ops yet |

## BunCraft's renderer

A stroke is a polyline of `Point { x, y, pressure }`. The renderer walks each
segment and stamps a disc every `0.55 × radius` pixels. Radius and alpha follow
pressure, so a tapered stroke is simply a pressure envelope.

Two stamp paths:

- **r ≤ 26 px**: direct premultiplied-alpha pixel blend. This is the hot path;
  a typical ink stamp touches ~25 pixels.
- **r > 26 px**: tiny-skia path fill (needed for 700 px background washes).

Airbrushes use a single radial gradient per stamp instead of stacked discs,
which removes banding and keeps a 760 px wash to one fill.

Why stamps instead of one filled outline per stroke? Pressure varies *along*
the stroke; a single `stroke-width` cannot express it, and SVG cannot either.
The SVG export therefore writes a centreline with a representative width, while
the PNG keeps the full pressure envelope. This is a documented trade-off, not
an accident.

## Measured performance (this machine, debug build)

| Operation | Time |
|---|---|
| generate 135 strokes / 28k points | 6 ms |
| render 1200×1600 PNG, 135 strokes | ~13 s debug |
| render 1200×1600 PNG, release | see `performance_analysis.md` |

The debug number is dominated by unoptimised pixel loops; the release build is
the relevant figure and is recorded in `performance_analysis.md`.

## Missing compared with the studied engines

- Boolean path operations (union/difference) for clean lineart intersections.
- True pressure-sensitive brush textures (only procedural grain today).
- HPGL/G-code output (SVG is the only vector export).
- Path simplification for plotters (Douglas–Peucker).
