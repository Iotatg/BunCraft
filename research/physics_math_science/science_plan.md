# Science wiring plan

`crates/buncraft-science/` contains real, tested maths. This file is the
sequence that connects it to the drawing pipeline. It exists because
**foundations that are not called are still, functionally, absent**.

## Ground rules

1. **One step per change set.** Each step must compile, pass `cargo test
   --release`, and leave the CLI able to draw before the next step starts.
2. **Additive first.** New functions (`mix_linear`, `sample_by_arclen`, …) land
   beside the old ones. Old behaviour is only retired once the new path has a
   passing test and a visual check.
3. **No new external dependencies.** `buncraft-science` is zero-dependency on
   purpose. Upstream `palette`/`kurbo`/`avian`/`rapier` are **not** added. The
   geometry and physics engines are edition 2024 with MSRV 1.84–1.89, while
   BunCraft declares `rust-version = "1.80"` (`Cargo.toml:17`). `palette`
   (MSRV 1.71) is compatible but is still not used, to keep the maths
   independently testable. All four remain read-only references.
4. **Determinism is preserved.** `buncraft-anime` has an `is_deterministic`
   test (same `prompt`/`seed`/`speed`/`correct` → identical JSON). Changing
   colour maths changes *pixels*, but must never introduce randomness or
   platform-dependent ordering.
5. **No claim without a test.** Every step below lists a measurable acceptance
   criterion. If it is not asserted, it is not done.

## Current state (baseline)

| Item | Value | Evidence |
|---|---|---|
| Workspace crates | 7 | `Cargo.toml:3-11` |
| Tests passing | 67 | `cargo test --release` |
| `buncraft-science` tests | 18 | `cargo test -p buncraft-science` |
| Consumers of `buncraft-science` | **1** (`buncraft-core`) | `crates/buncraft-core/Cargo.toml:13` |
| Colour mixing | `mix_linear` at the 3 additive-light sites; encoded `mix` retained for the 2 absorptive shadow sites | `crates/buncraft-core/src/stroke.rs:91-130` |
| Curve sampling | exact arc length via cubic-Bézier conversion + `inv_arclen` | `crates/buncraft-core/src/geom.rs` (`catmull_rom_arclen*`) |
| Shading | hand-authored constants | `crates/buncraft-anime/src/lib.rs:1513-1637` |
| Hair dynamics | none (static curves) | `crates/buncraft-anime/src/lib.rs:699-732` |

---

## Step 1 — Linear-light colour mixing (highest visual payoff, lowest risk)

**Why.** `Rgba::mix` interpolates gamma-encoded bytes. Light adds in linear
radiance, so a 0.8 white mix over silver hair is measurably dimmer than it
should be. This is the single change that most affects the "angel ring"
highlight and every shadow tone.

**Change.**

- `crates/buncraft-core/Cargo.toml`: add
  `buncraft-science = { workspace = true }`.
- `crates/buncraft-core/src/stroke.rs`: add

  ```rust
  pub fn mix_linear(self, other: Rgba, t: f32) -> Rgba
  pub fn luminance_linear(self) -> f32
  ```

  implemented as `LinRgb::from_srgb_u8(...).mix(..., t as f64).to_srgb_u8()`.
  Keep `mix` and `luminance` untouched — `luminance` is a Rec.601 luma proxy
  used as a *threshold* (`lib.rs:1476`), and its 0.72 cut-off was tuned on that
  quantity. Changing the threshold's definition would silently move the
  highlight; that is a separate decision.
- `crates/buncraft-anime/src/lib.rs`: switch the five `mix(` call sites
  (`1417`, `1431`, `1477`, `1479`, `1607` at time of writing) to `mix_linear`.

**Acceptance.**

- `mix_linear(black, white, 0.5)` encodes to `187±2`, not `128`; the test
  records both values.
- `luminance_linear(silver 210,215,228) ≈ 0.653 ± 0.005` (linear Rec.709),
  while the old gamma luma is `≈ 0.834` — the test asserts they differ.
- `mix_linear` agrees with `palette::LinSrgb` within `1/255` per channel
  (checked once, offline; the dependency is not added).
- All 67 existing tests still pass; `is_deterministic` still passes.

**Visual check.** Re-render the silver-haired portrait and compare the angel
ring against the previous PNG. The band should read as a band, not a smudge.

**Status: DONE (2026-11-16).** `Rgba::mix_linear` / `Rgba::luminance_linear`
added to `crates/buncraft-core/src/stroke.rs`, with four new unit tests
(`linear_mix_is_brighter_than_encoded_mix`, `linear_mix_preserves_alpha_linearly`,
`linear_luminance_differs_from_encoded_luma`, `linear_mix_endpoints_are_exact`).
Measured: black→white at `t = 0.5` is `187` (encoded mix gives `128`);
mid-grey `128` has linear luminance `0.2158` vs encoded luma `0.502`. The three
*additive* sites in `buncraft-anime` now use `mix_linear`; the two *absorptive*
sites (iris shade, hair shadow) deliberately keep encoded `mix` — see the
"shadows stay absorptive" note below.

The visual check exposed a second, non-colour problem: once the highlight was
bright enough to see, the old single continuous band read as a **headband**.
It is now three arched glints with gaps of bare hair, and the tone rule has
three luminance bands instead of two (light → 80 % white, mid-tone → 42 % white,
dark → cool tint). Pink hair no longer turns blue-grey.

**Lesson recorded:** linear-light mixing is only correct for *added* light.
Applying it to shadow lerps lightens them, because linear interpolation weights
the dark end less than the eye expects. Shadow needs multiplication by an
albedo/visibility factor, which is Step 4.

---

## Step 2 — Linear-light compositing in the rasteriser

**Status: DEFERRED (2026-11-16), with reason.**

Only `circle()` (r ≤ 26) blends pixels by hand (`lib.rs:343-347`). Every larger
stamp, every path fill and every gradient goes through `tiny-skia`'s
`fill_path`, which composites internally in 8-bit sRGB (`circle_path`
`lib.rs:352-366`, `soft_circle` `lib.rs:370-400`). Converting *only* `circle()`
to linear light would create a visible seam at the `r = 26` switch
(`lib.rs:312-315`) — the same class of discontinuity the audit already flags
for alpha. A correct fix requires compositing the whole layer in a linear
buffer, which `tiny-skia`'s `Pixmap` (8-bit sRGB, premultiplied) does not
provide.

Options, in order of cost:

1. Replace `tiny-skia` compositing with a manual f32 layer buffer and use
   `tiny-skia` only for coverage masks. Large change; touches every draw call.
2. Render at 16-bit per channel and convert at the end. Still sRGB-space
   blending, so it does not fix the physics.
3. Accept sRGB compositing and document it. This is the current state.

Decision: keep option 3 until the renderer is restructured. Step 1 already
fixes the *authored* colour mixes, which is where the visible error was
largest. The remaining error is in per-stamp overlap and anti-aliased edges.

**Why the original plan was wrong.** It assumed `circle()` was the only
compositing path. The audit's §3.1 looked at `circle()` and missed that
`circle_path` and `soft_circle` delegate to `tiny-skia`. Recorded here so the
next attempt does not repeat the mistake.

---

## Step 2b — Linear-light compositing (original design, kept for reference)

**Why.** `circle()` blends `cr·a + dst·(1−a)` directly on `u8` sRGB bytes
(`crates/buncraft-render/src/lib.rs:343-347`). Every stamp therefore composites
in the wrong space, and the result depends on how many stamps overlap — dark
edges get disproportionately dark.

**Change.**

- Add 256-entry decode and 4096-entry encode lookup tables in
  `buncraft-render` (built once, `OnceLock`), so per-pixel cost stays close to
  the current integer path.
- In `circle()` and `circle_path()`, decode source and destination, blend in
  linear radiance, encode on write.
- Fix the alpha inconsistency: `to_color` multiplies `color.a`
  (`lib.rs:58-61`) while `circle()` ignores it (`lib.rs:326`), producing a
  discontinuity at `r = 26` (`lib.rs:312-315`). Both paths must apply alpha the
  same way.

**Acceptance.**

- A 50 % white stamp over mid-grey produces a linear-light result; the test
  asserts the value differs from the gamma-space result by more than 20/255.
- Two radii straddling `r = 26` (`25` and `27`) produce centre pixels within
  `1/255` of each other.
- The five existing render tests (`renders_ink_pixels`,
  `svg_keeps_single_point_airbrush_dabs`, `clipped_stroke_stays_inside_mask`,
  `replay_starts_blank`, `svg_contains_path`) still pass.

**Risk.** This is the slowest step per pixel. Measure `buncraft draw` wall time
before and after; if it regresses more than ~15 %, keep the LUT path but skip
decode/encode when `a == 1.0` (opaque stamps are a no-op blend).

---

## Step 3 — Arc-length curve sampling

**Why.** `catmull_rom` gives every anchor segment the same 10 samples in `t`.
Where the curve is fast, samples are sparse; where it is slow, they bunch. Pen
speed, taper and grain are all functions of distance travelled, so this makes
them wrong.

**Change.**

- Convert each uniform Catmull–Rom segment to the equivalent cubic Bézier:

  ```text
  b0 = p1
  b1 = p1 + (p2 − p0) / 6
  b2 = p2 − (p3 − p1) / 6
  b3 = p2
  ```

  This is exact for α = 0 Catmull–Rom; it is not an approximation.
- In `crates/buncraft-core/src/geom.rs`, add `catmull_rom_arclen(points,
  samples_per_segment, accuracy)` that builds these `CubicBez` segments and
  calls `sample_by_arclen`. Keep `catmull_rom` as-is.
- Switch `Stroke::from_control_points` (`stroke.rs:306`) and
  `from_closed_control_points` (`stroke.rs:332`) to the new function.

**Acceptance.**

- A deliberately uneven anchor chain (short–long–short segments) sampled at 64
  points has adjacent arc-length ratio `max/min < 1.02`; the old uniform-`t`
  sampler is `> 3.0` on the same input. The test asserts both numbers.
- A straight line stays exactly straight and keeps its endpoints.
- Closed curves still close: first and last sample within `1e-4`.
- Existing geometry tests (`catmull_passes_through_anchors`,
  `closed_curve_is_closed`, `resample_keeps_endpoints`,
  `polygon_area_of_square`) still pass.

**Visual check.** Re-render all three portraits. Long hair strands should have
evenly spaced grain and taper; sharp bends should no longer show a "kink" where
samples bunch.

**Status: DONE (2026-11-16).** Added `cr_cubics` (exact uniform Catmull–Rom →
cubic Bézier conversion), `cubic_lengths`, `sample_cubics`,
`catmull_rom_arclen` and `catmull_rom_arclen_spacing` in
`crates/buncraft-core/src/geom.rs`. `Stroke::from_control_points` and
`from_closed_control_points` now sample the true spline by arc length at the
brush's stamp spacing, instead of tessellating at 10/12 points per segment and
then chord-resampling. `CubicBez::inv_arclen_with_total` was added to
`buncraft-science` so a segment's total length is not recomputed for every
sample.

Measured on a 4-anchor circle with 20°/50°/20° spacing: adjacent-sample
ratio `1.0002` in the interior, `< 1.02` including segment boundaries, versus
`> 2.0` for the uniform-`t` sampler. Draw time stayed at ~0.47 s per portrait
(no regression; the removed double sampling offsets the quadrature cost).

**Test lesson recorded:** an arc-length sampler is not required to land *on*
an anchor — the anchor can fall between two samples. The test asserts the
curve passes within one sample spacing of it, not through it.

---

## Step 4 — Physical shading from a light direction

**Why.** `stage_shading()` places shadows at fixed pixel offsets and
`highlight_band()` draws a parabola with a power curve. There is no light
direction, no surface normal, no irradiance. The result is makeup, not optics.

**Change.**

- Add `crates/buncraft-anime/src/shading.rs`:

  ```rust
  pub struct Light { pub dir: Vec2, pub irradiance: f32, pub ambient: f32 }
  pub fn face_normal(x: f32, y: f32, face: &Ellipse) -> [f32; 3]
  pub fn shade_face(light: &Light, n: [f32; 3], albedo: Rgba) -> Rgba
  ```

  `face_normal` uses a **hemisphere approximation** over the face ellipse:
  `n = (dx, dy, sqrt(max(0, 1 − dx² − dy²)))`. This is an explicit 2.5D model,
  not a 3D mesh — document it as such.
- `stage_shading()` calls `shade_face` for the broad shadow; the hard-edged
  cast shadow remains a separate stylised layer (anime shading is not
  physically based and should not pretend to be).
- `highlight_band()` keeps its silhouette-following shape but takes its colour
  from `light::cook_torrance` instead of a fixed `mix` factor.

**Acceptance.**

- A unit sphere lit head-on has brightness following `cos θ` within 2 % at 9
  sample angles (analytic Lambert).
- Rotating `light.dir` by 180° swaps the bright and dark sides: the correlation
  between the two luminance maps is `< −0.9`.
- `cook_torrance` is energy-bounded: a discrete integral over the hemisphere is
  `≤ 1 + 1e-3`.
- All existing tests pass; the `is_deterministic` test still passes.

**Honesty note.** This is a 2.5D approximation. It must not be described as
"3D rendering" or "global illumination". There is no ray tracing, no occlusion
map, no subsurface scattering.

---

## Step 5 (optional) — XPBD hair strands

**Why.** Twin tails and side strands are static curves. `buncraft-science::xpbd`
already has `Particle`, `DistanceConstraint`, `HairStrand`, `step`, `tip` and
`max_stretch`, with 5 passing tests.

**Change.**

- Add `crates/buncraft-anime/src/dynamics.rs` behind
  `#[cfg(feature = "hair_dynamics")]`, default **off**, so default output is
  byte-identical.
- Build one `HairStrand` per tail, pin the root at the scalp anchor, set
  gravity + a wind vector, and step at a fixed `1/120 s` with 4 substeps.
- Render the strand by sampling `tip()` / link positions into the existing
  `Stroke` pipeline; do **not** change the static path when the feature is off.

**Acceptance.**

- A single pendulum (`L = 1 m`, `g = 9.81`) has a period within 2 % of
  `2π√(L/g)`.
- Total energy drift over 10 s is `< 1 %`.
- Halving `Δt` changes the result by a factor `< 0.3` (convergence).
- `--features hair_dynamics` and default both pass `cargo test --release`;
  default output is unchanged.

---

## What this plan does *not* do

- It does not add 3D perspective, poses or 3/4 views. That needs a camera and
  projection matrix, which is a separate milestone.
- It does not add path booleans, tessellated stroke outlines or plotter export.
- It does not train a stroke model. No weights exist; none are fabricated.
- It does not make the evaluator aesthetic. The score remains geometric.

## Dependency decision (recorded)

The audit (`notes/buncraft_audit.md` §9) recommended adding `palette`, `kurbo`
and `avian` directly. That was written before the MSRV check. The geometry and
physics crates require Rust 1.84–1.89 and edition 2024; BunCraft pins 1.80.
(`palette` is the exception at 1.71, but it is still not used.) Rather than
raise the toolchain floor for a drawing app, the maths was reimplemented
independently in `buncraft-science` and verified against the upstream source
with `path:line` citations. If the MSRV is raised later, the upstream crates can
replace the internal implementations behind the same function signatures.
