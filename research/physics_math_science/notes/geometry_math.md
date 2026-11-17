# Geometry and linear algebra — verified notes

Scope: what the cloned geometry / linear-algebra repositories actually contain,
with `path:line` evidence, and what BunCraft now uses versus still lacks.
All paths are relative to `/workspace/BunCraft`. Repositories were **read, not
built or executed**.

## 1. kurbo — 2D curve geometry

Version `0.13.1`, `edition = "2024"`, `rust-version = "1.85"`
(`research/upstream/kurbo/Cargo.toml:10,12,15`).

| Capability | Evidence |
|---|---|
| Gauss–Legendre arc-length quadrature | `research/upstream/kurbo/kurbo/src/param_curve.rs:71` (`fn gauss_arclen`) |
| Arc-length trait entry point | `.../param_curve.rs:88` (`fn arclen`) |
| Inverse arc length | `.../param_curve.rs:98` (`fn inv_arclen`) |
| Cubic quadrature core | `.../cubicbez.rs:616` (`arclen_quadrature_core`) |
| Adaptive recursion with error estimate | `.../cubicbez.rs:628` (`arclen_rec`) |
| `CubicBez::arclen` implementation | `.../cubicbez.rs:673` |

The approach is: estimate `∫₀¹ |C′(t)| dt` with Gauss–Legendre, compare the
whole-segment estimate against the sum of two half-segment estimates, and
subdivide until the difference is below the requested accuracy (with a depth
cap). This is exactly the method reimplemented independently in
`crates/buncraft-science/src/curve.rs` (`gauss8`, `arclen_refine`).

## 2. nalgebra — dense linear algebra

Version `0.35.0`, `edition = "2024"`, `rust-version = "1.89.0"`
(`research/upstream/nalgebra/Cargo.toml:3,14,15`).

Decompositions are exposed as methods on matrices in
`research/upstream/nalgebra/src/linalg/decomposition.rs`:

| Method | Line |
|---|---|
| `lu` | `:48` |
| `qr` | `:57` |
| `svd` | `:80` |
| `cholesky` | `:257` |
| `symmetric_eigen` | `:334` |

Implementations live in sibling files (`lu.rs`, `qr.rs`, `cholesky.rs`,
`svd.rs`, `symmetric_eigen.rs`, `full_piv_lu.rs`, `col_piv_qr.rs`, …) under
`research/upstream/nalgebra/src/linalg/`. Low-dimensional fixed-size matrices
(`SMatrix`) are the relevant part for 2D/3D transforms; BunCraft does not need
the dynamic-size machinery.

## 3. faer-rs — dense/sparse factorisations

Version `0.24.4`, `edition = "2021"`, `rust-version = "1.84.0"`
(`research/upstream/faer-rs/faer/Cargo.toml:3,4,11`). Solver entry points are
in `research/upstream/faer-rs/faer/src/linalg/solvers.rs`. The workspace splits
traits (`faer-traits`), macros (`faer-macros`) and the main crate (`faer`).

## 4. lyon — tessellation

Version `1.0.23`, `edition = "2024"`
(`research/upstream/lyon/crates/tessellation/Cargo.toml:3,11`).

- Fill tessellation: `research/upstream/lyon/crates/tessellation/src/fill.rs`.
  Tolerance is a first-class option (`:544`), defaulted at `:576`, and consumed
  by the event queue at `:598` and `:623`. The tolerance is halved when
  intersections are handled (`:785-786`).
- Stroke tessellation: `.../src/stroke.rs`.
- Geometry output: `.../src/geometry_builder.rs`.
- The relevant capability for BunCraft is **path boolean / stroke outline**
  generation, which is still missing (see §6).

## 5. ndarray — N-dimensional arrays

Version `0.17.2`, `edition = "2021"`, `rust-version = "1.87"`
(`research/upstream/ndarray/Cargo.toml:4,5,6`). Linear-algebra bindings live in
`research/upstream/ndarray/src/linalg/impl_linalg.rs` and
`.../src/linalg/mod.rs`. Relevant for batched stroke fitting, not for the
current single-portrait pipeline.

## 6. What BunCraft had, and what changed

**Before (still present as `catmull_rom`):**
`crates/buncraft-core/src/geom.rs` sampled a uniform Catmull–Rom spline at a
fixed number of points per segment (`catmull_rom`, ~`:124-143`), then
`resample` (~`:185-213`) spaced points by **chord length of that coarse
polyline**. Neither step knew the analytic speed `|C′(t)|`, so a long segment
and a short segment received the same sample count.

**Now (2026-11-16):**
`crates/buncraft-core/src/geom.rs` adds:

- `cr_cubics` — exact conversion of each uniform Catmull–Rom segment to a cubic
  Bézier: `b0 = p1`, `b1 = p1 + (p2−p0)/6`, `b2 = p2 − (p3−p1)/6`, `b3 = p2`.
  This is an identity, not an approximation (the endpoint derivatives match).
- `cubic_lengths` — per-segment arc length + cumulative table, using
  `CubicBez::arclen(0.02)`.
- `sample_cubics` — walks the chain and emits points at constant arc length,
  using `CubicBez::inv_arclen_with_total`.
- `catmull_rom_arclen` and `catmull_rom_arclen_spacing` — public entry points.
- `Stroke::from_control_points` / `from_closed_control_points` now call
  `catmull_rom_arclen_spacing` directly.

**Still missing:** curvature, tangent/normal accessors on `Vec2`; exact
curve bounding boxes; OBB / convex hull; path booleans; tessellated stroke
outlines. `Bounds` remains a sampled AABB (`geom.rs:69-117`).

## 7. Licence position

| Repo | Files | SPDX |
|---|---|---|
| kurbo | `LICENSE-APACHE`, `LICENSE-MIT` | Apache-2.0 OR MIT |
| nalgebra | `LICENSE` | Apache-2.0 |
| faer-rs | `LICENSE` | MIT |
| lyon | `LICENSE-APACHE`, `LICENSE-MIT` | Apache-2.0 OR MIT (API says NOASSERTION) |
| ndarray | `LICENSE-APACHE`, `LICENSE-MIT` | MIT OR Apache-2.0 |

All are compatible with BunCraft's MIT workspace (`Cargo.toml:16`). No code was
copied; `buncraft-science` is an independent implementation with its own tests.

## 8. MSRV conflict (the reason these are not dependencies)

| Crate | Required Rust | BunCraft |
|---|---:|---:|
| nalgebra 0.35.0 | 1.89.0 | **1.80** |
| kurbo 0.13.1 | 1.85 | **1.80** |
| faer 0.24.4 | 1.84.0 | **1.80** |
| ndarray 0.17.2 | 1.87 | **1.80** |
| lyon 1.0.23 | edition 2024 (≥1.85) | **1.80** |

Adding any of them directly would raise BunCraft's toolchain floor by 4–9
releases. The maths was therefore reimplemented in a zero-dependency crate and
cross-checked against these sources.

## 9. Actionable next steps

1. Add `curvature`, `tangent`, `normal` to `buncraft-core::geom` using
   `CubicBez::curvature` / `deriv` (already implemented and tested in
   `buncraft-science`).
2. Use curvature to modulate brush width (high curvature → slightly thinner),
   which is a real pen behaviour rather than a constant taper.
3. Add exact Bézier bounding boxes (control-point hull is a conservative
   bound) before adding OBB.
4. Defer path booleans until the renderer can consume tessellated outlines.
5. Revisit upstream dependencies only if the MSRV is raised.
