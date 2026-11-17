# Limitations

BunCraft is an MVP. This page is the short, user-facing version of
`research/research_gaps.md`.

## Not implemented

- **Trained neural drawing model.** The GRU inference code exists; no weights
  ship, and the CLI never uses it to draw.
- **Anime stroke dataset.** No public corpus provides ordered anime strokes
  with pressure.
- **Pose and composition.** Front-facing bust only.
- **Hands, full body, perspective.**
- **Animation export.** Timing and replay exist; MP4/GIF, keyframes and
  in-betweens do not.
- **Manga panels and speech bubbles.**
- **Path booleans.** Intersections are layered, not merged.
- **Plotter output.** SVG only; no HPGL/G-code.

## Physics / maths: partly applied

`crates/buncraft-science/` is a zero-dependency, independently implemented crate
with sRGB/Oklab linear-light colour, analytic Bézier arc length and inverse arc
length, an XPBD distance/bend constraint solver, and Lambert/GGX/Cook–Torrance
shading. It has 18 passing unit tests. Two of its four areas are now wired in:

- **Applied:** additive highlights mix in **linear light**
  (`Rgba::mix_linear`), and every spline stroke is sampled by **true arc
  length**, so stamp spacing follows distance travelled rather than the curve
  parameter.
- **Not applied:** shading and highlights are still hand-authored constants, not
  computed from a light direction and surface normal. Hair strands are static
  geometry, not simulated. The rasteriser still composites in **sRGB**, because
  `tiny-skia` blends internally and exposes no linear buffer; converting only
  the small-disc path would create a seam at its `r = 26` switch.
- **Shadow tone is still absorptive.** Iris and hair-root shadows use the
  encoded `Rgba::mix` toward near-black, because shadow is absorption, not
  added light. Multiplicative linear shading is the next step.

The upstream engines (`nalgebra`, `kurbo`, `rapier`, `avian`) were studied but
**not** compiled, executed, vendored, or linked. Their MSRV (1.84–1.89) exceeds
BunCraft's 1.80, so they cannot be direct dependencies. `palette` is the one
exception (MSRV 1.71); it is still not a dependency because the colour maths is
implemented in the zero-dependency science crate. No pretrained weights, no
fabricated simulation results, and no image-generation wrapper are involved.

## What the score means

`report.json → evaluation.score` is a **geometric consistency** score built from
symmetry and proportion errors. It is not an aesthetic judgement. A drawing can
score 91/100 and still look stiff.

## What "AI" means here

The current artist is a constructive rule system with a deterministic seed. It
is not a neural network. The architecture is model-ready (stroke model trait,
stroke-5 dataset I/O, checkpoints), but the intelligence is anatomical
construction, not learned weights. Calling it an AI artist today would be
marketing, not fact.
