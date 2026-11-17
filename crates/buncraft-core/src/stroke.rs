//! Stroke representation: the atomic unit of BunCraft's drawing language.
//!
//! A stroke is a polyline of pressure-annotated points plus the brush that
//! rendered it. Curves are flattened at authoring time with a Catmull-Rom
//! spline, which keeps rendering cheap and makes replay deterministic: the
//! same document always produces the same pixels.

use serde::{Deserialize, Serialize};

use buncraft_science::color::LinRgb;

use crate::geom::{self, Bounds, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushKind {
    /// Graphite: grainy, semi-transparent, pressure sensitive.
    Pencil,
    /// Technical pen / fineliner: crisp, near-opaque, thin.
    Ink,
    /// Brush pen: strong taper, organic width.
    Brush,
    /// Marker: wide, flat, translucent, used for flat colour.
    Marker,
    /// Airbrush: soft falloff, used for blush and gradients.
    Airbrush,
}

impl BrushKind {
    /// Nominal drawing speed in pixels per second, used by the scheduler.
    pub fn base_speed(self) -> f32 {
        match self {
            BrushKind::Pencil => 1250.0,
            BrushKind::Ink => 720.0,
            BrushKind::Brush => 900.0,
            BrushKind::Marker => 1500.0,
            BrushKind::Airbrush => 1600.0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BrushKind::Pencil => "pencil",
            BrushKind::Ink => "ink",
            BrushKind::Brush => "brush",
            BrushKind::Marker => "marker",
            BrushKind::Airbrush => "airbrush",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn with_alpha(self, a: f32) -> Self {
        Self {
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ..self
        }
    }

    pub fn alpha_f(self) -> f32 {
        self.a as f32 / 255.0
    }

    /// Perceived brightness in `0.0..=1.0` (Rec. 601 weights).
    ///
    /// Shading and highlighting need this to pick a tone step that is actually
    /// visible: a 40% white mix does nothing to near-white hair, and a 40%
    /// black mix does nothing to near-black hair.
    pub fn luminance(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    pub fn mix(self, other: Rgba, t: f32) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Rgba::new(
            m(self.r, other.r),
            m(self.g, other.g),
            m(self.b, other.b),
            m(self.a, other.a),
        )
    }

    /// Mix two colours in **linear light**, then re-encode to sRGB bytes.
    ///
    /// This is the physically correct operation for adding light: radiance is
    /// linear, so a 50% mix of black and white is ~0.735 in encoded sRGB, not
    /// 0.5. [`Rgba::mix`] interpolates the gamma-encoded bytes instead, which
    /// darkens every blend.
    ///
    /// Alpha is *coverage*, not radiance, so it is interpolated linearly.
    pub fn mix_linear(self, other: Rgba, t: f32) -> Rgba {
        let tf = t.clamp(0.0, 1.0);
        let rgb = LinRgb::from_srgb_u8(self.r, self.g, self.b)
            .mix(
                LinRgb::from_srgb_u8(other.r, other.g, other.b),
                tf as f64,
            )
            .to_srgb_u8();
        let a = (self.a as f32 + (other.a as f32 - self.a as f32) * tf).round() as u8;
        Rgba::new(rgb[0], rgb[1], rgb[2], a)
    }

    /// Relative luminance Y in linear light (Rec.709 / sRGB primaries).
    ///
    /// Distinct from [`Rgba::luminance`], which is a Rec.601 luma proxy on
    /// gamma-encoded bytes. For mid-grey (128) this returns ≈0.216, while the
    /// encoded luma is ≈0.502.
    pub fn luminance_linear(self) -> f32 {
        LinRgb::from_srgb_u8(self.r, self.g, self.b).luminance() as f32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brush {
    pub kind: BrushKind,
    /// Nominal diameter in pixels.
    pub size: f32,
    pub opacity: f32,
    pub color: Rgba,
    /// 0 = constant width, 1 = strongly tapered ends.
    pub taper: f32,
    /// Per-stamp positional noise in pixels (hand tremor).
    pub jitter: f32,
}

impl Brush {
    pub fn new(kind: BrushKind, size: f32, color: Rgba) -> Self {
        let (opacity, taper, jitter) = match kind {
            BrushKind::Pencil => (0.55, 0.25, 0.45),
            BrushKind::Ink => (0.96, 0.15, 0.15),
            BrushKind::Brush => (0.9, 0.6, 0.3),
            BrushKind::Marker => (0.35, 0.0, 0.2),
            BrushKind::Airbrush => (0.22, 0.0, 0.6),
        };
        Self {
            kind,
            size,
            opacity,
            color,
            taper,
            jitter,
        }
    }

    pub fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }

    pub fn with_size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    pub fn with_color(mut self, c: Rgba) -> Self {
        self.color = c;
        self
    }

    pub fn with_jitter(mut self, j: f32) -> Self {
        self.jitter = j;
        self
    }

    pub fn with_taper(mut self, t: f32) -> Self {
        self.taper = t;
        self
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    /// 0..1, drives width and alpha.
    pub pressure: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, pressure: f32) -> Self {
        Self { x, y, pressure }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// The human drawing workflow, in the order a real artist works.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Composition,
    Rough,
    Construction,
    Face,
    Eyes,
    NoseMouth,
    Hair,
    Body,
    Clothing,
    Lineart,
    Color,
    Shading,
    Correction,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Stage::Composition => "composition",
            Stage::Rough => "rough sketch",
            Stage::Construction => "construction lines",
            Stage::Face => "facial structure",
            Stage::Eyes => "eyes",
            Stage::NoseMouth => "nose & mouth",
            Stage::Hair => "hair",
            Stage::Body => "body & anatomy",
            Stage::Clothing => "clothing",
            Stage::Lineart => "clean lineart",
            Stage::Shading => "shadows & lighting",
            Stage::Color => "colouring",
            Stage::Correction => "inspection & corrections",
        }
    }

    /// Stage ordering used by the timeline UI.
    pub fn index(self) -> u8 {
        match self {
            Stage::Composition => 0,
            Stage::Rough => 1,
            Stage::Construction => 2,
            Stage::Face => 3,
            Stage::Eyes => 4,
            Stage::NoseMouth => 5,
            Stage::Hair => 6,
            Stage::Body => 7,
            Stage::Clothing => 8,
            Stage::Lineart => 9,
            Stage::Color => 10,
            Stage::Shading => 11,
            Stage::Correction => 12,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stroke {
    pub id: u32,
    pub layer: u8,
    pub stage: Stage,
    pub brush: Brush,
    pub points: Vec<Point>,
    /// Human readable purpose, e.g. "left upper eyelid".
    pub role: String,
    /// Deterministic seed for texture / tremor.
    pub seed: u64,
    /// Optional polygon mask (canvas space) applied to this stroke only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<Vec<Vec2>>,
}

impl Stroke {
    pub fn new(
        id: u32,
        layer: u8,
        stage: Stage,
        brush: Brush,
        points: Vec<Point>,
        role: impl Into<String>,
        seed: u64,
    ) -> Self {
        Self {
            id,
            layer,
            stage,
            brush,
            points,
            role: role.into(),
            seed,
            clip: None,
        }
    }

    pub fn length(&self) -> f32 {
        self.points
            .windows(2)
            .map(|w| w[0].pos().dist(w[1].pos()))
            .sum()
    }

    pub fn bounds(&self) -> Bounds {
        let mut b = Bounds::empty();
        for p in &self.points {
            b.include(p.pos());
        }
        b
    }

    pub fn with_clip(mut self, clip: Vec<Vec2>) -> Self {
        self.clip = Some(clip);
        self
    }

    /// Build a smooth stroke from control points.
    ///
    /// `spacing` is the stamp distance; smaller means smoother and slower.
    pub fn from_control_points(
        control: &[Vec2],
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
        seed: u64,
    ) -> Self {
        // Sample the true spline by arc length: stamps land at a constant
        // distance travelled, not at a constant curve parameter.
        let spacing = (brush.size * 0.45).clamp(1.0, 5.0);
        let sampled = geom::catmull_rom_arclen_spacing(control, spacing, false);
        let n = sampled.len();
        let mut points = Vec::with_capacity(n);
        for (i, p) in sampled.iter().enumerate() {
            let t = if n <= 1 {
                1.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let pressure = pressure_profile(t, brush.taper);
            points.push(Point::new(p.x, p.y, pressure));
        }
        Stroke::new(0, layer, stage, brush, points, role, seed)
    }

    /// Build a closed stroke (ellipse-like shapes, silhouettes).
    pub fn from_closed_control_points(
        control: &[Vec2],
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
        seed: u64,
    ) -> Self {
        let spacing = (brush.size * 0.45).clamp(1.0, 5.0);
        let sampled = geom::catmull_rom_arclen_spacing(control, spacing, true);
        let n = sampled.len();
        let mut points = Vec::with_capacity(n);
        for (i, p) in sampled.iter().enumerate() {
            let t = i as f32 / (n.max(2) - 1) as f32;
            points.push(Point::new(p.x, p.y, pressure_profile(t, brush.taper)));
        }
        Stroke::new(0, layer, stage, brush, points, role, seed)
    }

    /// Build a stroke that keeps hard corners (no spline smoothing).
    /// Used for garment necklines and other constructed edges.
    pub fn from_polyline(
        points: &[Vec2],
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
        seed: u64,
    ) -> Self {
        let spacing = (brush.size * 0.45).clamp(1.0, 5.0);
        let sampled = geom::resample(points, spacing);
        let n = sampled.len();
        let mut pts = Vec::with_capacity(n);
        for (i, p) in sampled.iter().enumerate() {
            let t = if n <= 1 {
                1.0
            } else {
                i as f32 / (n - 1) as f32
            };
            pts.push(Point::new(p.x, p.y, pressure_profile(t, brush.taper)));
        }
        Stroke::new(0, layer, stage, brush, pts, role, seed)
    }

    /// Straight line helper.
    pub fn line(
        a: Vec2,
        b: Vec2,
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
        seed: u64,
    ) -> Self {
        Stroke::from_control_points(&[a, b], brush, layer, stage, role, seed)
    }
}

/// Width/alpha envelope: thin at the ends when `taper > 0`.
pub fn pressure_profile(t: f32, taper: f32) -> f32 {
    let taper = taper.clamp(0.0, 1.0);
    let base = if taper <= 0.0 {
        1.0
    } else {
        let edge = (t / taper.max(0.08)).min(1.0);
        let edge_out = ((1.0 - t) / taper.max(0.08)).min(1.0);
        (edge.min(edge_out)).powf(0.55).max(0.12)
    };
    // a little organic variation in the middle of the stroke
    let wobble = 1.0 + 0.06 * (t * 17.0).sin();
    (base * wobble).clamp(0.05, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_spans_black_to_white() {
        assert!(Rgba::rgb(0, 0, 0).luminance() < 0.01);
        assert!(Rgba::rgb(255, 255, 255).luminance() > 0.99);
        // green is perceptually brighter than blue at the same value
        assert!(Rgba::rgb(0, 255, 0).luminance() > Rgba::rgb(0, 0, 255).luminance());
    }

    #[test]
    fn taper_thins_the_ends() {
        assert!(pressure_profile(0.0, 0.5) < pressure_profile(0.5, 0.5));
        assert!(pressure_profile(1.0, 0.5) < pressure_profile(0.5, 0.5));
    }

    #[test]
    fn stroke_length_matches_polyline() {
        let brush = Brush::new(BrushKind::Ink, 2.0, Rgba::rgb(0, 0, 0));
        let s = Stroke::line(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            brush,
            3,
            Stage::Lineart,
            "test",
            1,
        );
        assert!((s.length() - 100.0).abs() < 1.0);
        assert!(s.points.len() > 10);
    }

    #[test]
    fn alpha_is_clamped() {
        let c = Rgba::rgb(10, 20, 30);
        assert_eq!(c.with_alpha(2.0).a, 255);
        assert_eq!(c.with_alpha(-1.0).a, 0);
    }

    #[test]
    fn linear_mix_is_brighter_than_encoded_mix() {
        let black = Rgba::rgb(0, 0, 0);
        let white = Rgba::rgb(255, 255, 255);

        let encoded = black.mix(white, 0.5).r;
        let linear = black.mix_linear(white, 0.5).r;

        // The whole point: a 50% light mix is ~187 in sRGB, not 128.
        assert!(
            (encoded as i32 - 128).abs() <= 1,
            "encoded mix should be ~128, got {encoded}"
        );
        assert!(
            (linear as i32 - 187).abs() <= 2,
            "linear mix should be ~187, got {linear}"
        );
        assert!(linear as i32 - encoded as i32 > 50);
    }

    #[test]
    fn linear_mix_preserves_alpha_linearly() {
        let a = Rgba::new(10, 20, 30, 0);
        let b = Rgba::new(200, 210, 220, 255);
        // Alpha is coverage, not light, so it must stay at the arithmetic mean.
        assert_eq!(a.mix_linear(b, 0.5).a, 128);
        assert_eq!(a.mix_linear(b, 1.0).a, 255);
        assert_eq!(a.mix_linear(b, 0.0).a, 0);
    }

    #[test]
    fn linear_luminance_differs_from_encoded_luma() {
        let mid = Rgba::rgb(128, 128, 128);
        // Rec.709 luminance of linear mid-grey is ~0.216 ...
        assert!((mid.luminance_linear() - 0.2158).abs() < 0.002);
        // ... while the gamma-encoded Rec.601 luma is ~0.502.
        assert!((mid.luminance() - 0.502).abs() < 0.002);
        assert!((mid.luminance() - mid.luminance_linear()).abs() > 0.25);
    }

    #[test]
    fn linear_mix_endpoints_are_exact() {
        let a = Rgba::rgb(12, 200, 77);
        let b = Rgba::rgb(240, 30, 190);
        assert_eq!(a.mix_linear(b, 0.0), a);
        assert_eq!(a.mix_linear(b, 1.0), b);
    }
}
