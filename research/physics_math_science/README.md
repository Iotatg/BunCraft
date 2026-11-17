# Physics, mathematics and science — study index

This directory records a deliberate study of **real** geometry, colour science,
dynamics and lighting, done before wiring any of it into BunCraft. It exists
because "looks like physics" is not physics: a highlight drawn at a fixed pixel
offset is a makeup rule, not optics.

## What was done

- **11 upstream repositories cloned** (depth 1, single branch) into
  `research/upstream/` on 2026-11-15. Exact commits, sizes and licences:
  [`repo_manifest.md`](repo_manifest.md).
- **Source read with `path:line` citations.** No upstream repository was
  compiled or executed. The `run` evidence tag is unused.
- **Cross-checked formulas** by the lead agent: [`notes/verified_formulas.md`](notes/verified_formulas.md).
- **Three area notes**:
  - [`notes/geometry_math.md`](notes/geometry_math.md) — kurbo, nalgebra,
    faer-rs, lyon, ndarray.
  - [`notes/dynamics.md`](notes/dynamics.md) — XPBD, avian, rapier, parry,
    PositionBasedDynamics.
  - [`notes/color_light.md`](notes/color_light.md) — palette, colour-science.
- **A full audit of BunCraft's own code** for fake or missing maths:
  [`notes/buncraft_audit.md`](notes/buncraft_audit.md). It lists 16 gaps with
  evidence, plus a four-step minimal plan.
- **An independent, zero-dependency implementation** in
  `crates/buncraft-science/`: `color`, `curve`, `light`, `xpbd`. 18 unit tests.
- **A wiring plan with measurable acceptance criteria**:
  [`science_plan.md`](science_plan.md).

## What is verified, and how

| Claim | Tag | Evidence |
|---|---|---|
| sRGB EOTF/OETF constants | `source` | `research/upstream/palette/palette/src/encoding/srgb.rs:128-146` |
| Oklab M1/M2 and inverses | `source` | `research/upstream/palette/palette/src/oklab.rs:281-300,309-320` |
| Adaptive Gauss–Legendre arc length | `source` | `research/upstream/kurbo/kurbo/src/param_curve.rs:71,88,98`; `cubicbez.rs:616,628,673` |
| XPBD multiplier update | `source` | `research/upstream/PositionBasedDynamics/PositionBasedDynamics/XPBD.cpp:36-56` |
| avian compliance form | `source` | `research/upstream/avian/src/dynamics/solver/xpbd/mod.rs:424-426` |
| avian contacts are impulse-based, not XPBD | `source` | `research/upstream/avian/src/dynamics/solver/xpbd/mod.rs:1-5` |
| GJK / EPA entry points | `source` | `research/upstream/parry/src/query/gjk/gjk.rs:353`; `epa/epa2.rs:103,342` |
| CIEDE2000 / D-series illuminants | `source` | `research/upstream/colour/colour/difference/delta_e.py:666`; `colorimetry/illuminants.py:254` |
| No BRDF in palette or colour-science | `source` (negative) | `rg -i 'brdf\|cook.torrance\|ggx\|fresnel'` → no matches |
| BunCraft's own gaps | `source` | `notes/buncraft_audit.md` §1–§8 |
| Upstream code compiled/executed | **`none`** | not done, not claimed |

## Verified vs textbook

This distinction matters, because it is easy to imply that a formula came from
a repository when it did not.

| Maths | Origin |
|---|---|
| sRGB transfer, Oklab matrices | **Repository** (palette), constants cited |
| Adaptive arc length, inverse arc length | **Repository approach** (kurbo), independently implemented |
| XPBD compliance update | **Repository + paper** (PositionBasedDynamics, avian, Macklin et al. 2016) |
| GJK / EPA / SAT | **Repository** (parry), read only |
| Lambert, half-Lambert, GGX, Smith G, Fresnel–Schlick, Cook–Torrance | **Textbook / paper only** — *not* in either colour repository |
| Oklab ΔE (`delta_e_ok`) | Independent simplification; **not** CIEDE2000 |

## What BunCraft now uses

| Area | Status | Where |
|---|---|---|
| Linear-light colour mixing | **wired** for additive light | `crates/buncraft-core/src/stroke.rs` (`mix_linear`), `crates/buncraft-anime/src/lib.rs` |
| Arc-length spline sampling | **wired** for all spline strokes | `crates/buncraft-core/src/geom.rs` (`catmull_rom_arclen*`) |
| XPBD hair solver | implemented, tested, **not wired** | `crates/buncraft-science/src/xpbd.rs` |
| Lambert/GGX lighting | implemented, tested, **not wired** | `crates/buncraft-science/src/light.rs` |
| Linear-light raster compositing | **blocked** on `tiny-skia`'s internal sRGB blend | `crates/buncraft-render/src/lib.rs` |

Current test baseline: **75 tests passing** across 7 crates
(`cargo test --release`), of which 18 are in `buncraft-science`.

## Licence position

All cloned repositories are permissive: Apache-2.0, MIT, Apache-2.0 OR MIT, or
BSD-3-Clause (the Python colour library). No GPL/AGPL was cloned. No upstream
code was copied into the Rust crates; constants are cited and implementations
are independent. BunCraft is MIT (`Cargo.toml:16`). Details:
[`repo_manifest.md`](repo_manifest.md).

## Honest gaps

- **No upstream repository was built or run.** Claims about behaviour come from
  reading source and from BunCraft's own tests, not from executing the upstream
  code.
- **REA agent tooling was not installed or run.** All inspection was direct.
  GitHub's unauthenticated search API hit HTTP 403 rate limits mid-study, so
  repository discovery is not exhaustive.
- **Lighting is 2.5D at best.** There is no 3D mesh, no ray tracing, no
  occlusion map, no subsurface scattering. Any future `light::shade` wiring
  must be described as a hemisphere approximation.
- **The rasteriser is still sRGB.** Authored mixes are linear; per-stamp
  overlap and anti-aliased edges are not.
- **No trained stroke model, no aesthetic judgement, no 3/4 view, no animation
  pipeline, no vector booleans, no plotter export.** These remain true; see
  `research/research_gaps.md`.
