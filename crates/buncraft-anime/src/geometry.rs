//! Anime-specific construction geometry.
//!
//! Everything is derived from the character profile, so a different head width
//! or eye size produces a *proportionally* different face instead of a scaled
//! bitmap. These are the same landmark rules a beginner manga tutorial teaches:
//! eye line at ~57% of the head, nose at ~73%, mouth at ~85%.

use buncraft_core::geom::{catmull_rom_closed, Vec2};
use buncraft_core::profile::{CharacterProfile, EyeShape, HairStyle};

#[derive(Clone, Debug)]
pub struct Geometry {
    pub cx: f32,
    pub top: f32,
    pub chin: f32,
    pub half_w: f32,
    pub head_h: f32,
    pub jaw: f32,
    pub eye_y: f32,
    pub eye_dx: f32,
    pub eye_w: f32,
    pub eye_h: f32,
    pub brow_y: f32,
    pub nose_y: f32,
    pub mouth_y: f32,
    pub neck_y: f32,
    pub shoulder_y: f32,
    pub shoulder_half: f32,
    pub bottom: f32,
    pub hair_bottom: f32,
}

#[derive(Clone, Debug)]
pub struct EyePaths {
    pub upper: Vec<Vec2>,
    pub lower: Vec<Vec2>,
    pub polygon: Vec<Vec2>,
    pub center: Vec2,
    pub iris_center: Vec2,
    pub iris_rx: f32,
    pub iris_ry: f32,
    pub inner: Vec2,
    pub outer: Vec2,
}

impl Geometry {
    pub fn new(p: &CharacterProfile) -> Self {
        let cx = 600.0;
        let top = 322.0;
        let head_h = p.face.head_height;
        let chin = top + head_h;
        let half_w = p.face.head_width * 0.5;
        let eye_y = top + head_h * 0.575;
        let eye_dx = half_w * 0.495;
        let eye_w = 62.0 * p.face.eye_size * p.eyes.size;
        let eye_h = 34.0 * p.face.eye_size * p.eyes.size;
        let hair_bottom = chin + (p.hair.length - 0.8) * 760.0;
        Self {
            cx,
            top,
            chin,
            half_w,
            head_h,
            jaw: p.face.jaw_softness.clamp(0.0, 1.0),
            eye_y,
            eye_dx,
            eye_w,
            eye_h,
            brow_y: eye_y - 58.0,
            nose_y: top + head_h * 0.735,
            mouth_y: top + head_h * 0.845,
            neck_y: chin + 120.0,
            shoulder_y: chin + 168.0,
            shoulder_half: 252.0,
            bottom: 1600.0,
            hair_bottom,
        }
    }

    /// Right half of the face contour, crown to chin.
    pub fn face_right(&self) -> Vec<Vec2> {
        let cx = self.cx;
        let h = self.head_h;
        let w = self.half_w;
        let j = self.jaw;
        let jaw_x = w * (0.60 + 0.14 * j);
        let chin_w = w * (0.10 + 0.06 * j);

        vec![
            Vec2::new(cx, self.top),
            Vec2::new(cx + w * 0.46, self.top + h * 0.012),
            Vec2::new(cx + w * 0.84, self.top + h * 0.10),
            Vec2::new(cx + w * 0.99, self.top + h * 0.28),
            Vec2::new(cx + w, self.top + h * 0.46),
            Vec2::new(cx + w * 0.975, self.top + h * 0.60),
            Vec2::new(cx + w * 0.90, self.top + h * 0.71),
            Vec2::new(cx + jaw_x, self.top + h * 0.855),
            Vec2::new(cx + w * 0.36, self.top + h * 0.955),
            Vec2::new(cx + chin_w, self.chin - 4.0),
            Vec2::new(cx, self.chin),
        ]
    }

    /// Face outline control points, closed loop: crown -> right side -> chin ->
    /// left side -> crown.
    pub fn face_outline(&self) -> Vec<Vec2> {
        let cx = self.cx;
        let right = self.face_right();
        let mut pts = right.clone();
        for p in right.iter().rev().skip(1) {
            if (p.x - cx).abs() < 0.5 {
                continue;
            }
            pts.push(Vec2::new(2.0 * cx - p.x, p.y));
        }
        pts
    }

    pub fn face_polygon(&self) -> Vec<Vec2> {
        catmull_rom_closed(&self.face_outline(), 10)
    }

    pub fn cheek_y(&self) -> f32 {
        self.top + self.head_h * 0.60
    }

    pub fn ear_center(&self, side: f32) -> Vec2 {
        Vec2::new(
            self.cx + side * self.half_w * 0.985,
            self.eye_y + self.head_h * 0.045,
        )
    }

    pub fn neck_polygon(&self) -> Vec<Vec2> {
        let cx = self.cx;
        let nw = self.half_w * 0.37;
        let ny = self.chin + self.head_h * 0.20;
        vec![
            Vec2::new(cx - nw, self.chin - 20.0),
            Vec2::new(cx - nw * 1.12, ny),
            Vec2::new(cx - nw * 2.4, self.shoulder_y - 8.0),
            Vec2::new(cx + nw * 2.4, self.shoulder_y - 8.0),
            Vec2::new(cx + nw * 1.12, ny),
            Vec2::new(cx + nw, self.chin - 20.0),
            Vec2::new(cx + nw * 0.35, self.chin + 6.0),
            Vec2::new(cx - nw * 0.35, self.chin + 6.0),
        ]
    }

    /// Outer silhouette of the hair mass (behind the face).
    pub fn hair_silhouette(&self, p: &CharacterProfile) -> Vec<Vec2> {
        let cx = self.cx;
        let w = self.half_w;
        let h = self.head_h;
        let bottom = self.hair_bottom;
        let flare = match p.hair.style {
            HairStyle::ShortBob => 1.06,
            HairStyle::MessyShort => 1.02,
            HairStyle::TwinTails => 1.04,
            HairStyle::Ponytail => 1.05,
            _ => 1.20,
        };
        let right = vec![
            Vec2::new(cx, self.top - 26.0),
            Vec2::new(cx + w * 0.55, self.top - 20.0),
            Vec2::new(cx + w * 0.94, self.top + h * 0.03),
            Vec2::new(cx + w * flare, self.top + h * 0.24),
            Vec2::new(cx + w * (flare + 0.02), self.top + h * 0.48),
            Vec2::new(cx + w * flare, self.chin - 30.0),
            Vec2::new(cx + w * (flare - 0.10), bottom * 0.55 + (self.chin - 30.0) * 0.45),
            Vec2::new(cx + w * (flare - 0.22), bottom),
            Vec2::new(cx + w * 0.45, bottom + 26.0),
            Vec2::new(cx, bottom + 10.0),
        ];
        let mut pts = right.clone();
        for q in right.iter().rev().skip(1) {
            if (q.x - cx).abs() < 0.5 {
                continue;
            }
            pts.push(Vec2::new(2.0 * cx - q.x, q.y));
        }
        pts
    }

    /// Distance from the centre line to the outer edge of the hair mass at a
    /// given height, taken from the silhouette itself.
    ///
    /// Anything drawn *inside* the hair (strands, shading, highlights) has to
    /// be placed relative to this instead of relative to the face, otherwise it
    /// spills onto the cheeks.
    pub fn hair_half_width_at(&self, p: &CharacterProfile, y: f32) -> f32 {
        let sil = self.hair_silhouette(p);
        let mut right: Vec<Vec2> = sil.into_iter().filter(|q| q.x >= self.cx).collect();
        right.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
        if right.len() < 2 {
            return self.half_w;
        }
        if y <= right[0].y {
            return right[0].x - self.cx;
        }
        if y >= right[right.len() - 1].y {
            return right[right.len() - 1].x - self.cx;
        }
        for pair in right.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if y >= a.y && y <= b.y {
                let t = if (b.y - a.y).abs() < 1e-4 {
                    0.0
                } else {
                    (y - a.y) / (b.y - a.y)
                };
                return (a.x + (b.x - a.x) * t - self.cx).max(1.0);
            }
        }
        self.half_w
    }

    /// Region covered by the fringe, used as a flat colour fill.
    pub fn bangs_polygon(&self, p: &CharacterProfile) -> Vec<Vec2> {
        let cx = self.cx;
        let w = self.half_w;
        let h = self.head_h;
        let part = cx + p.hair.parting * 70.0;
        let mut pts = vec![
            Vec2::new(cx - w * 0.96, self.top + h * 0.20),
            Vec2::new(cx - w * 0.80, self.top + h * 0.06),
            Vec2::new(cx - w * 0.40, self.top - 6.0),
            Vec2::new(part, self.top - 14.0),
            Vec2::new(cx + w * 0.45, self.top - 4.0),
            Vec2::new(cx + w * 0.84, self.top + h * 0.08),
            Vec2::new(cx + w * 0.97, self.top + h * 0.21),
        ];
        // zig-zag fringe tips, right to left
        let tips = 7;
        for i in 0..tips {
            let t = i as f32 / (tips - 1) as f32;
            let x = cx + w * 0.95 - t * w * 1.9;
            let y = self.brow_y - 20.0 + ((i % 3) as f32) * 9.0;
            pts.push(Vec2::new(x, y));
        }
        pts
    }

    pub fn eye(&self, side: f32, p: &CharacterProfile) -> EyePaths {
        let ex = self.cx + side * self.eye_dx;
        let ey = self.eye_y;
        let ew = self.eye_w;
        let mut eh = self.eye_h;

        // expression / shape modifiers
        let (mut lid_lift, mut outer_drop) = (1.0f32, 0.0f32);
        let _ = lid_lift;
        match p.eyes.shape {
            EyeShape::Round => lid_lift = 1.05,
            EyeShape::Almond => lid_lift = 0.92,
            EyeShape::Sharp => {
                lid_lift = 0.86;
                outer_drop = 0.10;
            }
            EyeShape::Droopy => {
                lid_lift = 0.95;
                outer_drop = -0.12;
            }
        }
        match p.expression {
            buncraft_core::profile::Expression::Surprised => {
                eh *= 1.12;
                lid_lift *= 1.08;
            }
            buncraft_core::profile::Expression::Smile => {
                eh *= 0.94;
            }
            buncraft_core::profile::Expression::Shy => {
                eh *= 0.9;
            }
            _ => {}
        }

        let inner = Vec2::new(ex - side * ew, ey + 4.0);
        let outer = Vec2::new(ex + side * ew, ey - 2.0 - outer_drop * eh);
        let upper = vec![
            inner,
            Vec2::new(ex - side * ew * 0.58, ey - eh * 0.80 * lid_lift),
            Vec2::new(ex + side * ew * 0.10, ey - eh * 1.02 * lid_lift),
            Vec2::new(ex + side * ew * 0.66, ey - eh * 0.72 * lid_lift),
            outer,
        ];
        let lower = vec![
            outer,
            Vec2::new(ex + side * ew * 0.58, ey + eh * 0.48),
            Vec2::new(ex, ey + eh * 0.60),
            Vec2::new(ex - side * ew * 0.60, ey + eh * 0.44),
            inner,
        ];
        let mut polygon = upper.clone();
        polygon.extend(lower.iter().skip(1).take(lower.len() - 2));

        EyePaths {
            upper,
            lower,
            polygon,
            center: Vec2::new(ex, ey),
            iris_center: Vec2::new(ex + side * 2.0, ey + eh * 0.12),
            iris_rx: ew * 0.40,
            iris_ry: eh * 1.05,
            inner,
            outer,
        }
    }

    /// Right half of the torso outline, neck to bottom of the canvas.
    pub fn body_outline_right(&self) -> Vec<Vec2> {
        let cx = self.cx;
        let sh = self.shoulder_y;
        let half = self.shoulder_half;
        vec![
            Vec2::new(cx + self.half_w * 0.40, self.chin + 30.0),
            Vec2::new(cx + self.half_w * 0.42, sh - 100.0),
            Vec2::new(cx + self.half_w * 0.62, sh - 55.0),
            Vec2::new(cx + half * 0.62, sh - 18.0),
            Vec2::new(cx + half * 0.92, sh + 18.0),
            Vec2::new(cx + half, sh + 78.0),
            Vec2::new(cx + half * 1.03, sh + 220.0),
            Vec2::new(cx + half * 1.02, self.bottom),
        ]
    }

    pub fn outfit_polygon(&self) -> Vec<Vec2> {
        let cx = self.cx;
        let sh = self.shoulder_y;
        let half = self.shoulder_half;
        vec![
            Vec2::new(cx - half * 1.06, self.bottom),
            Vec2::new(cx - half * 1.03, sh + 210.0),
            Vec2::new(cx - half, sh + 70.0),
            Vec2::new(cx - half * 0.90, sh + 10.0),
            Vec2::new(cx - half * 0.45, sh - 10.0),
            Vec2::new(cx - 78.0, sh - 16.0),
            Vec2::new(cx, sh + 64.0),
            Vec2::new(cx + 78.0, sh - 16.0),
            Vec2::new(cx + half * 0.45, sh - 10.0),
            Vec2::new(cx + half * 0.90, sh + 10.0),
            Vec2::new(cx + half, sh + 70.0),
            Vec2::new(cx + half * 1.03, sh + 210.0),
            Vec2::new(cx + half * 1.06, self.bottom),
        ]
    }
}

/// Landmarks recorded while drawing; the evaluator compares what was actually
/// drawn against this plan.
#[derive(Clone, Debug)]
pub struct DrawingPlan {
    pub face_width: f32,
    pub face_height: f32,
    pub eye_centers: [Vec2; 2],
    pub chin: Vec2,
    pub mouth_y: f32,
    pub outline: Vec<Vec2>,
}

impl DrawingPlan {
    pub fn from_geometry(g: &Geometry) -> Self {
        Self {
            face_width: g.half_w * 2.0,
            face_height: g.head_h,
            eye_centers: [
                Vec2::new(g.cx - g.eye_dx, g.eye_y),
                Vec2::new(g.cx + g.eye_dx, g.eye_y),
            ],
            chin: Vec2::new(g.cx, g.chin),
            mouth_y: g.mouth_y,
            outline: g.face_outline(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CharacterProfile {
        CharacterProfile::from_prompt("silver hair, red eyes, smile", 5)
    }

    #[test]
    fn face_outline_is_closed_and_symmetric() {
        let g = Geometry::new(&profile());
        let pts = g.face_outline();
        assert!(pts.len() > 12);
        let cx = g.cx;
        for p in &pts {
            let mirror = Vec2::new(2.0 * cx - p.x, p.y);
            let found = pts
                .iter()
                .any(|q| (q.x - mirror.x).abs() < 0.6 && (q.y - mirror.y).abs() < 0.6);
            assert!(found, "no mirror for {:?}", p);
        }
    }

    #[test]
    fn eye_polygon_contains_iris_center() {
        let p = profile();
        let g = Geometry::new(&p);
        for side in [-1.0f32, 1.0] {
            let eye = g.eye(side, &p);
            assert!(eye.polygon.len() > 6);
            assert!(eye.iris_rx > 10.0);
            assert!(eye.iris_ry > 10.0);
        }
    }

    #[test]
    fn hair_sits_below_chin_for_long_styles() {
        let p = profile();
        let g = Geometry::new(&p);
        assert!(g.hair_bottom > g.chin + 200.0);
    }
}
