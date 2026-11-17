# Research gaps

These are the things BunCraft **cannot** do yet. They are listed so that no
reader mistakes the MVP for the final system.

## 1. No trained stroke model

`buncraft-brain::model` implements a GRU with the SketchRNN interface and
checkpoint I/O. There is no trained checkpoint, and the CLI never uses it.
A random GRU draws noise; shipping one would be dishonest.

Needed: an anime stroke corpus (see 2), a training loop, and an evaluation that
measures stroke-level reconstruction rather than pixel FID.

## 2. No anime stroke dataset

QuickDraw is genuine human stroke data but doodle-scale (≤25 strokes). Anime
lineart datasets are images. `text-to-sketch` vectorises lineart, which yields
*extracted* paths with inferred order — useful, but not recorded hand motion.

Needed: a licensed corpus with stroke order and pressure, or a documented
extraction pipeline whose ordering is validated against human drawings.

## 3. No aesthetic judgement

The evaluator measures symmetry, proportion, chin placement, eye spacing and
coverage. It cannot say whether a face is beautiful, expressive or on-model.
The 90/100 score is a *geometric consistency* score, not an art score.

## 4. Limited pose and composition

The generator draws a front-facing bust. There is no 3/4 view, profile, hand,
full body, or perspective. `CharacterProfile` has no pose field.

## 5. No animation or manga pipeline

Timing and replay exist. Keyframes, in-betweens, camera, speech bubbles, panel
layout and MP4 export do not.

## 6. No vector boolean operations

Lineart intersections are drawn over each other. There is no path union or
erase, so hair over the face is layered, not cut.

## 7. No text encoder

Prompts are parsed by keyword matching (hair colour, style, eye colour,
expression, outfit). There is no semantic understanding, no composition
reasoning, and unknown words are recorded as notes rather than interpreted.

## 8. No pen-plotter output

SVG exists; HPGL and G-code do not. Path simplification for physical pens is
missing.

## 9. Real physics / maths: partly wired, partly still on the shelf

`crates/buncraft-science/` contains tested, independent implementations of
linear-light sRGB/Oklab colour, analytic cubic-Bézier arc length and inverse arc
length, an XPBD constraint solver (distance + simplified bend, substepped
semi-implicit Euler), and Lambert / GGX / Cook–Torrance shading.

**Wired in (2026-11-16):**

- `crates/buncraft-core` depends on `buncraft-science` and exposes
  `Rgba::mix_linear` / `Rgba::luminance_linear`. Additive highlights in
  `buncraft-anime` (hair angel ring, iris glow) now mix in linear radiance.
- `Stroke::from_control_points` / `from_closed_control_points` sample the exact
  spline by arc length (`catmull_rom_arclen_spacing`), so stamp spacing follows
  distance travelled. Uniform-`t` sampling remains available as
  `catmull_rom` for callers that want it.

**Still unwired:**

- `light::shade` is not called; `stage_shading()` and `highlight_band()` still
  use hand-authored constants. There is no light direction or surface normal.
- `xpbd::HairStrand` is not driven by the timeline; twin tails are static.
- The rasteriser still composites in sRGB: `tiny-skia`'s `fill_path` blends
  internally, so converting only the hand-written `circle()` path would create
  a seam at `r = 26`.
- Shadow tone still uses the encoded `Rgba::mix`, because shadow is absorption
  (albedo multiplication), not added light.

Needed: Step 4 (2.5D hemisphere normal + `light::shade`) and Step 5 (optional
feature-gated hair dynamics) in `physics_math_science/science_plan.md`. Upstream
geometry/physics engines were read (with `path:line` evidence) but not built,
executed, or linked; their edition-2024 / MSRV-1.84+ requirement conflicts with
BunCraft's 1.80, so they remain references rather than dependencies. `palette`
(MSRV 1.71) is compatible but is not used, because the colour maths lives in the
zero-dependency science crate.

## 10. REA agent tooling was not run

The master brief mentioned `npx rea-agents setup` where supported. It was not
installed or executed. All eleven upstream repositories were inspected directly
from their cloned source instead. This is a tooling gap in the research process,
not a claim that REA was used. GitHub's unauthenticated search API hit HTTP 403
rate limits mid-study, so repository discovery is incomplete; no credentials
were used or stored in the deliverables.

## What is genuinely implemented

- Deterministic stroke-based drawing with 13 stages.
- Pressure, taper, grain, hatching, airbrush, marker and ink brushes.
- Character profiles with cross-drawing consistency.
- Self-evaluation and a correction pass based on measured error.
- PNG, SVG, JSON timeline, frame sequence, browser replay, HTTP API.
- `buncraft-science`: linear-light sRGB/Oklab with ΔE_ok, analytic Bézier
  arc length / inverse arc length, XPBD hair-strand solver, Lambert/GGX shading
  — 18 unit tests, all passing. Colour mixing and arc-length sampling are wired
  in; lighting and XPBD are not (see gap 9).
- 75 unit tests across geometry, rendering, scheduling, models, datasets and
  the science crate; `cargo test --release` is green across all 7 crates.
