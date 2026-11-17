# BunCraft

**An AI that doesn't just generate art. It learns to draw.**

BunCraft is a stroke-based anime/manga drawing engine written in **pure Rust**.
It does not wrap an image generator. It plans a character, draws construction
lines, inks the face, lays in flat colour, shades it, checks its own proportions
and corrects the contour — and every one of those steps is an observable,
replayable drawing action.

```bash
cargo build --release
./target/release/buncraft draw \
  --prompt "silver haired girl with red eyes, shy smile, school uniform" \
  --seed 7 --out assets --speed 4
```

```
BunCraft — Character-0007 — anime portrait
  drew in 6 ms · rendered in 305 ms · timeline 29.4s
  136 strokes · 20 fills · 28259 points · 12 stages
  self-check 90.6/100 · symmetry 2.07px · proportion 0.5% · coverage 31.2%
```

Open `assets/preview.html` to watch the drawing happen stroke by stroke.

---

## What is actually implemented

| Area | Status |
|---|---|
| Stroke model (pressure, taper, 5 brush kinds) | working |
| Raster renderer (tiny-skia, deterministic) | working |
| SVG + PNG export | working |
| 13-stage human drawing workflow | working |
| Anime face / eyes / hair / body / clothing construction | working for bust portraits |
| Character profiles + persistent visual memory | working |
| Self-evaluation (symmetry, proportion, chin, eye spacing, coverage) | working |
| Correction pass triggered by measured error | working |
| Replay timeline + browser player + HTTP API | working |
| Stroke-5 dataset import, GRU inference, checkpoint I/O | working code, **no trained weights** |
| Pose variation, full body, hands | not implemented |
| Manga panels, animation export | not implemented |

The honest limitation list is in [`docs/limitations.md`](docs/limitations.md).
The engine does **not** claim to be a trained artist yet: it is a constructive
artist with a model-ready architecture.

## Why it is not an image-generator wrapper

A drawing is a `Document` containing ordered `Action`s:

```rust
enum Action {
    Stroke(Stroke),   // polyline + pressure + brush + layer + stage + role
    Fill(FillRegion), // polygon + colour + optional clip
}
```

The renderer is one projection of that data. SVG is another. The timeline player
is a third. Nothing in `buncraft-core` can even open a PNG.

Each stroke records *what it was for* (`"left upper eyelid"`, `"fringe (ink)"`,
`"correction: right contour"`), which is what makes self-evaluation and
correction possible.

## The drawing pipeline

| # | Stage | What it emits |
|---|---|---|
| 1 | composition | axes, head volume, shoulder line |
| 2 | rough sketch | loose head shape, gesture, hair mass |
| 3 | construction | face cross, eye boxes, nose/mouth guides |
| 4 | facial structure | jaw halves, ears, neck |
| 5 | eyes | lids, iris, pupil, lash, brow |
| 6 | nose & mouth | expression-dependent |
| 7 | hair | silhouette, tails, fringe strands, side locks |
| 8 | body | torso, collarbones |
| 9 | clothing | collar or hood, ribbon, folds |
| 10 | clean lineart | second, confident ink pass |
| 11 | colour | flat fills, clipped iris, highlights |
| 12 | shading | hatching, cast shadow, blush |
| 13 | correction | only if the evaluator found an error |

Timing is derived from path length: a long line genuinely takes longer, and
`--speed 4` is a timelapse rather than a fake reveal.

## Self-evaluation

After drawing, the brain compares the emitted contour with the construction
plan:

- **symmetry** — mean nearest distance between the right contour and the
  mirrored left contour;
- **proportion** — measured face width vs planned width;
- **chin** — vertical deviation from the planned chin line;
- **eye spacing** — measured vs planned inter-eye distance;
- **coverage** — fraction of the canvas touched by ink.

If an error exceeds tolerance, `apply_corrections` adds a real second pass and
the report marks the issue `corrected`. The score is geometric consistency, not
an art critic.

## Character consistency

```rust
let profile = CharacterProfile::from_prompt("silver hair, violet eyes", 5);
memory.insert(profile);
let again = brain.draw_known("Character-0005", Some("smile"));
```

A profile fixes head proportions, eye shape/colour, hair style/colour/length,
expression and outfit. The same profile with a different seed produces the same
character with a different hand tremor.

## CLI

```bash
buncraft draw --prompt "..." --seed 7 --out assets --speed 4
buncraft inspect assets/timeline.json
buncraft bench --runs 5
buncraft stages
```

## HTTP API + replay

```bash
cargo run --release -p buncraft-api -- 8080
curl -s localhost:8080/api/draw -H 'content-type: application/json' \
  -d '{"prompt":"pink twin tails, blue eyes, cheerful, hoodie","seed":12}'
```

See [`docs/api.md`](docs/api.md).

## Repository layout

```
crates/
  buncraft-core/    geometry, strokes, actions, documents, profiles, memory
  buncraft-render/  tiny-skia rasteriser, SVG, coverage
  buncraft-anime/   construction geometry + 13-stage artist
  buncraft-brain/   prompt→profile, evaluation, correction, GRU, dataset
  buncraft-science/ zero-dependency maths: linear-light colour, Bézier arc
                    length, XPBD constraints, Lambert/GGX shading
  buncraft-cli/     buncraft binary + replay player
  buncraft-api/     axum server
research/           13-repo desk study + 11 cloned physics/maths repos
docs/               architecture, drawing model, CLI, API, limits, roadmap
assets/             generated examples + timelines
```

## Research

The design came from a documented desk study of 13 repositories, including
Sketch-HARP, Sketchformer, text-to-sketch, Anime2Sketch, DrawingBotV3,
doodlewithAI, whiteboard-video-engine and OpenToonz MCP. Every claim is tagged
`meta` / `readme` / `source` / `run`; nothing is assumed from a repository name.

A second study went further: **11 geometry, colour and physics repositories
were cloned** and read with `path:line` citations (kurbo, nalgebra, faer-rs,
lyon, ndarray, palette, colour-science, avian, rapier, parry,
PositionBasedDynamics). None were compiled or executed. Start at
[`research/physics_math_science/README.md`](research/physics_math_science/README.md).

Licenses were checked before reuse: GPL-3.0 and unlicensed repositories were
read for ideas only, never copied. See
[`research/license_compliance.md`](research/license_compliance.md).

## Tests

```bash
cargo test --workspace
```

Covers geometry, spline interpolation, resampling, pressure envelopes,
scheduling, layer clipping, render determinism, replay, SVG output, prompt
parsing, evaluation, correction, GRU numerics, checkpoint validation, dataset
conversion and end-to-end drawing — plus the science crate: sRGB/Oklab round
trips, linear-light vs encoded mixing, Bézier arc length against closed forms,
arc-length sampling evenness, XPBD free fall and constraint convergence, and
Lambert/GGX energy bounds. **75 tests, all green.**

## License

MIT. No third-party source, weights, brushes or datasets are bundled.
