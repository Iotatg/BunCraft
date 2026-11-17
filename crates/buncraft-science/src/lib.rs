//! # buncraft-science
//!
//! Independent, dependency-free implementations of the *real* physics,
//! mathematics and colour/optics that BunCraft needs. It is deliberately a
//! separate crate: existing drawing crates keep working unchanged, and every
//! formula here can be unit-tested on its own before being wired into the
//! stroke/render pipeline.
//!
//! | Module | What it replaces / supplies | Primary evidence |
//! |---|---|---|
//! | [`color`] | gamma-space `Rgba::mix` → linear-light mixing, Oklab, ΔE_ok | `research/physics_math_science/notes/color_light.md` |
//! | [`curve`] | uniform-t Catmull-Rom sampling → analytic Bézier, curvature, adaptive arc length + inverse | `.../notes/geometry_math.md` |
//! | [`xpbd`] | no physics at all → substepped XPBD hair strands with compliance | `.../notes/dynamics.md` |
//! | [`light`] | hardcoded highlight colours → Lambert / GGX / Fresnel radiometry | `.../notes/verified_formulas.md` |
//!
//! All constants and equations are cross-checked against the 11 repositories
//! cloned in `research/upstream/` (see `.../repo_manifest.md`) and against the
//! original XPBD paper (Macklin, Müller & Chentanez, MiG '16, Eq. 18). No
//! third-party source file is copied into this crate.

pub mod color;
pub mod curve;
pub mod light;
pub mod xpbd;
