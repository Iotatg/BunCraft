//! Colour science, independent of the renderer.
//!
//! The key rule: light adds in **linear radiance**, not in gamma-encoded sRGB.
//! `buncraft-core::Rgba::mix` interpolates the encoded bytes, which is why a
//! 40% white mix over silver hair is nearly invisible. Everything here works in
//! linear light and only encodes at the display boundary.
//!
//! Constants verified against `upstream/palette/palette/src/encoding/srgb.rs:131-146`
//! and `upstream/palette/palette/src/oklab.rs:281-300`. This is an independent
//! implementation, not copied code.

/// IEC 61966-2-1 sRGB electro-optical transfer function (decode).
/// `c` is a gamma-encoded channel in 0..=1, result is linear radiance.
pub fn srgb_decode(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Inverse of [`srgb_decode`] (encode for display / 8-bit storage).
pub fn srgb_encode(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// A colour in **linear** sRGB (not gamma encoded).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinRgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl LinRgb {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0 };

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Build from 8-bit sRGB bytes, decoding through the transfer function.
    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(
            srgb_decode(r as f64 / 255.0),
            srgb_decode(g as f64 / 255.0),
            srgb_decode(b as f64 / 255.0),
        )
    }

    /// Encode back to 8-bit sRGB bytes, clamping out-of-gamut values.
    pub fn to_srgb_u8(self) -> [u8; 3] {
        let q = |x: f64| (srgb_encode(x.clamp(0.0, 1.0)) * 255.0).round().clamp(0.0, 255.0) as u8;
        [q(self.r), q(self.g), q(self.b)]
    }

    /// Physically correct additive mix of two light signals.
    pub fn mix(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
        )
    }

    /// Relative luminance Y of **linear** sRGB, using the Rec.709 / sRGB
    /// primaries: Y = 0.2126 R + 0.7152 G + 0.0722 B.
    ///
    /// Note: `buncraft-core::Rgba::luminance` uses the Rec.601 coefficients
    /// (0.299/0.587/0.114) applied to *gamma-encoded* bytes, which is a
    /// different quantity — a luma proxy, not luminance.
    pub fn luminance(self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

/// Oklab: a perceptually uniform space. Mixing here keeps hue from shifting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// linear sRGB → LMS' (M1) → cube root → Oklab (M2).
/// Matrix coefficients: `upstream/palette/palette/src/oklab.rs:281-300`.
pub fn linear_srgb_to_oklab(c: LinRgb) -> Oklab {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());
    Oklab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

/// Oklab → linear sRGB (M2⁻¹ then cube then M1⁻¹).
/// Coefficients: `upstream/palette/palette/src/oklab.rs:55-80, 306-320`.
pub fn oklab_to_linear_srgb(c: Oklab) -> LinRgb {
    let l_ = c.l + 0.3963377774 * c.a + 0.2158037573 * c.b;
    let m_ = c.l - 0.1055613458 * c.a - 0.0638541728 * c.b;
    let s_ = c.l - 0.0894841775 * c.a - 1.2914855480 * c.b;
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    LinRgb::new(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )
}

/// Perceptual distance in Oklab (ΔE_ok), used to decide whether a highlight
/// tone step is actually visible to a human.
pub fn delta_e_ok(a: Oklab, b: Oklab) -> f64 {
    let (dl, da, db) = (a.l - b.l, a.a - b.a, a.b - b.b);
    (dl * dl + da * da + db * db).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_function_roundtrip() {
        for i in 0..=20 {
            let x = i as f64 / 20.0;
            assert!((srgb_encode(srgb_decode(x)) - x).abs() < 1e-12, "x={x}");
            assert!((srgb_decode(srgb_encode(x)) - x).abs() < 1e-12, "x={x}");
        }
        assert_eq!(srgb_decode(0.0), 0.0);
        assert_eq!(srgb_decode(1.0), 1.0);
        // the well-known mid-grey value, ~0.214 linear
        assert!((srgb_decode(0.5) - 0.21404114048223255).abs() < 1e-9);
    }

    #[test]
    fn linear_light_mix_is_brighter_than_gamma_mix() {
        let half_linear = LinRgb::BLACK.mix(LinRgb::WHITE, 0.5).to_srgb_u8()[0] as f64 / 255.0;
        let half_gamma = 0.5; // what Rgba::mix(black, white, 0.5) produces
        assert!((half_linear - 0.735).abs() < 0.01, "got {half_linear}");
        assert!(half_linear - half_gamma > 0.2);
    }

    #[test]
    fn silver_hair_highlight_is_visible_in_linear_light() {
        let silver = LinRgb::from_srgb_u8(210, 215, 228); // BunCraft's silver hair
        let lit = silver.mix(LinRgb::WHITE, 0.8);
        let d = delta_e_ok(linear_srgb_to_oklab(silver), linear_srgb_to_oklab(lit));
        // A perceptual difference this large is a real highlight, not a smudge.
        assert!(d > 0.02, "ΔE_ok = {d}");
        let [r, g, b] = lit.to_srgb_u8();
        // 0.8 linear mix of silver→white lands at ~[247,248,250]; keep the
        // 0.8 factor aligned with BunCraft's current hi_color mixing.
        assert!(r >= 245 && g >= 245 && b >= 245, "got {r},{g},{b}");
    }

    #[test]
    fn oklab_roundtrip_and_anchors() {
        let white = linear_srgb_to_oklab(LinRgb::WHITE);
        // The M1/M2 coefficients are intentionally truncated to 10 decimals
        // (same as palette), so white lands within ~1e-7 rather than exactly 1.
        assert!((white.l - 1.0).abs() < 1e-7);
        assert!(white.a.abs() < 1e-7 && white.b.abs() < 1e-7);
        let black = linear_srgb_to_oklab(LinRgb::BLACK);
        assert!(black.l.abs() < 1e-9);
        for c in [
            LinRgb::from_srgb_u8(238, 158, 180),
            LinRgb::from_srgb_u8(42, 40, 50),
            LinRgb::from_srgb_u8(108, 148, 208),
            LinRgb::new(0.2, 0.7, 0.3),
        ] {
            let back = oklab_to_linear_srgb(linear_srgb_to_oklab(c));
            // M1/M2 and their inverses are separately truncated to 10 decimals,
            // so the round trip is accurate to ~1e-7 (well under one 8-bit step).
            assert!((back.r - c.r).abs() < 1e-6, "r: {:?} -> {:?}", c, back);
            assert!((back.g - c.g).abs() < 1e-6);
            assert!((back.b - c.b).abs() < 1e-6);
        }
    }
}
