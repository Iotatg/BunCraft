//! Radiometric shading primitives.
//!
//! These are *standard textbook* formulae, not code from any cloned repository.
//! Neither `palette` nor `colour-science/colour` implements a BRDF; they provide
//! colorimetry. BunCraft's current shading (`buncraft-anime/src/lib.rs` stage 12)
//! is pure empirical geometry with no surface normal, light direction,
//! irradiance or occlusion. This module supplies the missing physical quantities.
//!
//! Units: irradiance E [W·m⁻²], radiance L [W·m⁻²·sr⁻¹], albedo ρ ∈ [0,1].

pub type Vec3 = [f64; 3];

pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn norm(a: Vec3) -> Vec3 {
    let l = (dot(a, a)).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// Lambertian diffuse radiance: L = ρ/π · E · max(0, n·l).
/// The 1/π is the normalising constant of the Lambertian BRDF; without it
/// energy is not conserved.
pub fn lambert(albedo: f64, irradiance: f64, n_dot_l: f64) -> f64 {
    albedo / std::f64::consts::PI * irradiance * n_dot_l.max(0.0)
}

/// Valve's Half-Lambert wrap lighting: a soft terminator for cel-ish anime
/// shading. It is a *stylisation*, not a physically conservative BRDF.
pub fn half_lambert(albedo: f64, irradiance: f64, n_dot_l: f64) -> f64 {
    let w = (n_dot_l + 1.0) * 0.5;
    albedo / std::f64::consts::PI * irradiance * w * w
}

/// Trowbridge–Reitz / GGX normal distribution function.
/// D(h) = α² / (π ((n·h)²(α²−1) + 1)²), α = roughness².
/// D integrates to 1 over the hemisphere; that is the energy-conservation link.
pub fn ggx_ndf(n_dot_h: f64, alpha: f64) -> f64 {
    let a2 = alpha * alpha;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 / (std::f64::consts::PI * d * d)
}

/// Schlick's Fresnel approximation: F(θ) = F₀ + (1−F₀)(1−cosθ)⁵.
pub fn fresnel_schlick(f0: f64, cos_theta: f64) -> f64 {
    let m = (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5);
    f0 + (1.0 - f0) * m
}

/// Smith masking-shadowing G₁ for GGX (height-correlated approximation).
pub fn smith_g1(n_dot_v: f64, alpha: f64) -> f64 {
    let a2 = alpha * alpha;
    let c = n_dot_v.clamp(1e-6, 1.0);
    2.0 * c / (c + (a2 + (1.0 - a2) * c * c).sqrt())
}

/// Full specular lobe: Cook–Torrance D·G·F / (4 (n·l)(n·v)).
pub fn cook_torrance(
    n: Vec3,
    l: Vec3,
    v: Vec3,
    roughness: f64,
    f0: f64,
    irradiance: f64,
) -> f64 {
    let (n, l, v) = (norm(n), norm(l), norm(v));
    let ndl = dot(n, l);
    let ndv = dot(n, v);
    if ndl <= 0.0 || ndv <= 0.0 {
        return 0.0;
    }
    let h = norm([l[0] + v[0], l[1] + v[1], l[2] + v[2]]);
    let ndh = dot(n, h).max(0.0);
    let alpha = roughness * roughness;
    let d = ggx_ndf(ndh, alpha);
    let g = smith_g1(ndl, alpha) * smith_g1(ndv, alpha);
    let f = fresnel_schlick(f0, dot(h, v).max(0.0).min(1.0));
    d * g * f / (4.0 * ndl * ndv) * irradiance * ndl
}

/// A complete two-term shade: diffuse + specular, with an optional occlusion
/// factor in [0,1]. This is the function stage 12 should eventually call.
pub fn shade(n: Vec3, l: Vec3, v: Vec3, albedo: f64, roughness: f64, irradiance: f64, ao: f64) -> f64 {
    let diffuse = lambert(albedo, irradiance, dot(norm(n), norm(l)));
    let specular = cook_torrance(n, l, v, roughness, 0.04, irradiance);
    (diffuse + specular) * ao.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambert_is_one_sided_and_normalised() {
        assert_eq!(lambert(1.0, std::f64::consts::PI, 1.0), 1.0);
        assert_eq!(lambert(1.0, 100.0, -0.5), 0.0);
        assert!((lambert(0.5, std::f64::consts::PI, 1.0) - 0.5).abs() < 1e-12);
        assert!(half_lambert(1.0, std::f64::consts::PI, -1.0) < 1e-12);
        assert!(half_lambert(1.0, std::f64::consts::PI, 0.0) > 0.0);
    }

    #[test]
    fn ggx_peaks_at_the_normal_and_narrows_with_roughness() {
        assert!((ggx_ndf(1.0, 1.0) - 1.0 / std::f64::consts::PI).abs() < 1e-12);
        assert!(ggx_ndf(1.0, 0.1) > ggx_ndf(1.0, 1.0));
        assert!(ggx_ndf(0.5, 0.1) < ggx_ndf(1.0, 0.1));
    }

    #[test]
    fn fresnel_grazes_to_one() {
        assert!((fresnel_schlick(0.04, 1.0) - 0.04).abs() < 1e-12);
        assert!((fresnel_schlick(0.04, 0.0) - 1.0).abs() < 1e-12);
        assert!((smith_g1(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn shade_is_zero_in_shadow_and_finite() {
        let n = [0.0, 0.0, 1.0];
        let l = norm([1.0, 0.0, 1.0]);
        let v = [0.0, 0.0, 1.0];
        let s = shade(n, l, v, 0.8, 0.4, 3.0, 1.0);
        assert!(s.is_finite() && s > 0.0);
        assert_eq!(shade(n, l, v, 0.8, 0.4, 3.0, 0.0), 0.0);
        let back = [-1.0, 0.0, -1.0];
        assert!(shade(n, back, v, 0.8, 0.4, 3.0, 1.0) < 1e-12);
    }
}
