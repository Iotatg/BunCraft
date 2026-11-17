//! The anime artist: turns a character profile into a complete, ordered
//! sequence of drawing actions, the way a human illustrator builds a portrait.
//!
//! The generator is deliberately *constructive*: every line is derived from
//! anatomy landmarks, never from a reference bitmap. That is what allows the
//! same character to be re-drawn in another pose or expression and still look
//! like itself.

mod geometry;

pub use geometry::{DrawingPlan, EyePaths, Geometry};

use buncraft_core::document::{
    Action, Canvas, DocMeta, Document, Issue, LAYER_BACKGROUND, LAYER_CONSTRUCTION, LAYER_FLAT,
    LAYER_FX, LAYER_LINEART, LAYER_SHADOW,
};
use buncraft_core::geom::{ellipse, mirror_x, Vec2};
use buncraft_core::profile::{CharacterProfile, Expression, HairStyle, OutfitKind};
use buncraft_core::rng::Rng;
use buncraft_core::stroke::{Brush, BrushKind, Rgba, Stage, Stroke};

const C_SKETCH: Rgba = Rgba::new(116, 136, 168, 255);
const C_CONSTRUCTION: Rgba = Rgba::new(148, 168, 200, 255);
const C_INK: Rgba = Rgba::new(44, 40, 50, 255);
const C_SKIN: Rgba = Rgba::new(250, 228, 214, 255);
const C_SKIN_SHADOW: Rgba = Rgba::new(228, 188, 182, 255);
const C_SCLERA: Rgba = Rgba::new(252, 252, 255, 255);
const C_PUPIL: Rgba = Rgba::new(36, 32, 42, 255);
const C_LIP: Rgba = Rgba::new(212, 136, 138, 255);
const C_BLUSH: Rgba = Rgba::new(240, 148, 160, 255);

pub struct Artwork {
    pub document: Document,
    pub plan: DrawingPlan,
}

/// Draw a complete anime portrait from a profile.
pub fn draw(profile: &CharacterProfile) -> Artwork {
    Artist::new(profile).run()
}

struct Artist<'a> {
    p: &'a CharacterProfile,
    geo: Geometry,
    doc: Document,
    rng: Rng,
}

impl<'a> Artist<'a> {
    fn new(p: &'a CharacterProfile) -> Self {
        let geo = Geometry::new(p);
        let canvas = Canvas::new(1200, 1600);
        let meta = DocMeta::new(
            format!("{} — anime portrait", p.name),
            p.summary(),
            p.seed,
        );
        let mut doc = Document::new(canvas, meta);
        doc.profile = Some(p.clone());
        let rng = Rng::new(p.seed ^ 0xA5A5_1234_5678_9ABC);
        Self { p, geo, doc, rng }
    }

    fn run(mut self) -> Artwork {
        self.stage_composition();
        self.stage_rough();
        self.stage_construction();
        self.stage_face();
        self.stage_eyes();
        self.stage_nose_mouth();
        self.stage_hair();
        self.stage_body();
        self.stage_clothing();
        self.stage_lineart();
        self.stage_color();
        self.stage_shading();
        let plan = DrawingPlan::from_geometry(&self.geo);
        Artwork {
            document: self.doc,
            plan,
        }
    }

    // -- brushes ------------------------------------------------------------

    fn pencil(&self, size: f32, color: Rgba, alpha: f32) -> Brush {
        Brush::new(BrushKind::Pencil, size, color)
            .with_opacity(alpha)
            .with_jitter(0.55)
    }

    fn ink(&self, size: f32, color: Rgba) -> Brush {
        Brush::new(BrushKind::Ink, size, color).with_jitter(0.18)
    }

    fn pen(&self, size: f32, color: Rgba) -> Brush {
        Brush::new(BrushKind::Brush, size, color).with_jitter(0.25)
    }

    fn air(&self, size: f32, color: Rgba, alpha: f32) -> Brush {
        Brush::new(BrushKind::Airbrush, size, color)
            .with_opacity(alpha)
            .with_jitter(0.2)
    }

    // -- primitives ---------------------------------------------------------

    fn jitter(&mut self, pts: &[Vec2], amount: f32) -> Vec<Vec2> {
        if amount <= 0.0 {
            return pts.to_vec();
        }
        pts.iter()
            .map(|p| {
                Vec2::new(
                    p.x + self.rng.gauss() * amount,
                    p.y + self.rng.gauss() * amount,
                )
            })
            .collect()
    }

    fn curve(
        &mut self,
        pts: &[Vec2],
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: &str,
        jitter: f32,
    ) {
        let pts = self.jitter(pts, jitter);
        self.doc.stroke(&pts, brush, layer, stage, role);
    }

    fn closed(
        &mut self,
        pts: &[Vec2],
        brush: Brush,
        layer: u8,
        stage: Stage,
        role: &str,
        jitter: f32,
    ) {
        let pts = self.jitter(pts, jitter);
        self.doc.closed_stroke(&pts, brush, layer, stage, role);
    }

    fn fill(&mut self, poly: Vec<Vec2>, color: Rgba, layer: u8, stage: Stage, role: &str) {
        self.doc.fill(poly, color, layer, stage, role);
    }

    fn fill_clipped(
        &mut self,
        poly: Vec<Vec2>,
        color: Rgba,
        layer: u8,
        stage: Stage,
        role: &str,
        clip: Vec<Vec2>,
    ) {
        let id = self.doc.fill(poly, color, layer, stage, role);
        if let Some(Action::Fill(f)) = self.doc.actions.get_mut(id as usize - 1) {
            f.clip = Some(clip);
        }
    }

    // -- stage 1: composition ----------------------------------------------

    fn stage_composition(&mut self) {
        let (cx, top, ey, sy, h, w) = (
            self.geo.cx,
            self.geo.top,
            self.geo.eye_y,
            self.geo.shoulder_y,
            self.geo.head_h,
            self.geo.half_w,
        );
        let b = self.pencil(2.0, C_CONSTRUCTION, 0.30);
        self.curve(
            &[Vec2::new(cx, 170.0), Vec2::new(cx, 1570.0)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Composition,
            "vertical axis",
            0.0,
        );
        self.curve(
            &[Vec2::new(cx - w * 1.35, ey), Vec2::new(cx + w * 1.35, ey)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Composition,
            "eye line",
            0.0,
        );
        self.curve(
            &[Vec2::new(cx - w * 1.45, sy), Vec2::new(cx + w * 1.45, sy)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Composition,
            "shoulder line",
            0.0,
        );
        let head_volume = ellipse(Vec2::new(cx, top + h * 0.5), w * 1.10, h * 0.56, 18);
        self.closed(
            &head_volume,
            b,
            LAYER_CONSTRUCTION,
            Stage::Composition,
            "head volume",
            0.0,
        );
    }

    // -- stage 2: rough sketch ---------------------------------------------

    fn stage_rough(&mut self) {
        let b = self.pencil(3.2, C_SKETCH, 0.55);
        let outline = self.geo.face_outline();
        self.closed(
            &outline,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Rough,
            "loose head shape",
            7.0,
        );

        let (cx, top, chin, h, w) = (
            self.geo.cx,
            self.geo.top,
            self.geo.chin,
            self.geo.head_h,
            self.geo.half_w,
        );
        self.curve(
            &[
                Vec2::new(cx, chin + 40.0),
                Vec2::new(cx + w * 0.2, chin + 110.0),
                Vec2::new(cx + w * 0.55, chin + 175.0),
                Vec2::new(cx + w * 0.95, chin + 200.0),
            ],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Rough,
            "body gesture",
            6.0,
        );
        self.curve(
            &[
                Vec2::new(cx - w * 0.9, top + h * 0.15),
                Vec2::new(cx - w * 1.05, top + h * 0.7),
                Vec2::new(cx - w * 0.85, chin + 60.0),
            ],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Rough,
            "hair mass block-in (left)",
            8.0,
        );
        self.curve(
            &[
                Vec2::new(cx + w * 0.9, top + h * 0.15),
                Vec2::new(cx + w * 1.05, top + h * 0.7),
                Vec2::new(cx + w * 0.85, chin + 60.0),
            ],
            b,
            LAYER_CONSTRUCTION,
            Stage::Rough,
            "hair mass block-in (right)",
            8.0,
        );
    }

    // -- stage 3: construction lines ---------------------------------------

    fn stage_construction(&mut self) {
        let b = self.pencil(1.8, C_CONSTRUCTION, 0.42);
        let (cx, top, chin, ey, h, w) = (
            self.geo.cx,
            self.geo.top,
            self.geo.chin,
            self.geo.eye_y,
            self.geo.head_h,
            self.geo.half_w,
        );
        self.curve(
            &[Vec2::new(cx, top + h * 0.05), Vec2::new(cx, chin - 10.0)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Construction,
            "face centre line",
            0.0,
        );
        self.curve(
            &[Vec2::new(cx - w * 0.9, ey), Vec2::new(cx + w * 0.9, ey)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Construction,
            "eye guideline",
            0.0,
        );

        for side in [-1.0f32, 1.0] {
            let eye = self.geo.eye(side, self.p);
            let (x0, x1) = (
                eye.center.x - self.geo.eye_w,
                eye.center.x + self.geo.eye_w,
            );
            let (y0, y1) = (ey - self.geo.eye_h, ey + self.geo.eye_h);
            let box_pts = [
                Vec2::new(x0, y0),
                Vec2::new(x1, y0),
                Vec2::new(x1, y1),
                Vec2::new(x0, y1),
            ];
            self.closed(
                &box_pts,
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Construction,
                "eye box",
                0.6,
            );
        }

        self.curve(
            &[
                Vec2::new(cx + 6.0, self.geo.nose_y - 30.0),
                Vec2::new(cx + 6.0, self.geo.nose_y + 6.0),
            ],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Construction,
            "nose guide",
            0.5,
        );
        self.curve(
            &[
                Vec2::new(cx - 40.0, self.geo.mouth_y),
                Vec2::new(cx + 40.0, self.geo.mouth_y),
            ],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Construction,
            "mouth guide",
            0.5,
        );

        for side in [-1.0f32, 1.0] {
            let e = self.geo.ear_center(side);
            let ear = ellipse(e, 26.0, 40.0, 14);
            self.closed(
                &ear,
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Construction,
                "ear placement",
                0.8,
            );
        }
    }

    // -- stage 4: facial structure -----------------------------------------

    fn stage_face(&mut self) {
        let b = self.pencil(2.6, C_SKETCH, 0.72);
        let right = self.geo.face_right();
        // temple -> cheek -> jaw -> chin, as an artist would stroke it
        let right_curve = right[3..].to_vec();
        self.curve(
            &right_curve,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Face,
            "right face contour",
            2.2,
        );
        let left_curve = mirror_x(&right_curve, self.geo.cx);
        self.curve(
            &left_curve,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Face,
            "left face contour",
            2.2,
        );

        for side in [-1.0f32, 1.0] {
            let e = self.geo.ear_center(side);
            let ear = vec![
                Vec2::new(e.x - side * 6.0, e.y - 34.0),
                Vec2::new(e.x + side * 22.0, e.y - 12.0),
                Vec2::new(e.x + side * 16.0, e.y + 26.0),
                Vec2::new(e.x - side * 6.0, e.y + 34.0),
            ];
            self.curve(
                &ear,
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Face,
                "ear",
                1.2,
            );
        }

        let (cx, chin, nw) = (self.geo.cx, self.geo.chin, self.geo.half_w * 0.36);
        for side in [-1.0f32, 1.0] {
            self.curve(
                &[
                    Vec2::new(cx + side * nw, chin - 6.0),
                    Vec2::new(cx + side * nw * 1.15, chin + 70.0),
                    Vec2::new(cx + side * nw * 2.0, self.geo.shoulder_y - 40.0),
                ],
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Face,
                "neck",
                1.4,
            );
        }
    }

    // -- stage 5: eyes ------------------------------------------------------

    fn stage_eyes(&mut self) {
        for side in [-1.0f32, 1.0] {
            self.draw_eye(side, false);
        }
    }

    fn draw_eye(&mut self, side: f32, ink_pass: bool) {
        let eye = self.geo.eye(side, self.p);
        let layer = if ink_pass {
            LAYER_LINEART
        } else {
            LAYER_CONSTRUCTION
        };
        let stage = if ink_pass { Stage::Lineart } else { Stage::Eyes };

        let upper_brush = if ink_pass {
            self.ink(3.2, C_INK)
        } else {
            self.pencil(2.4, C_SKETCH, 0.75)
        };
        self.curve(
            &eye.upper,
            upper_brush,
            layer,
            stage,
            "upper eyelid",
            if ink_pass { 0.4 } else { 1.1 },
        );

        let lower_brush = if ink_pass {
            self.ink(1.8, C_INK)
        } else {
            self.pencil(1.5, C_SKETCH, 0.55)
        };
        self.curve(
            &eye.lower,
            lower_brush,
            layer,
            stage,
            "lower eyelid",
            0.7,
        );

        let iris = ellipse(eye.iris_center, eye.iris_rx, eye.iris_ry, 20);
        let iris_brush = if ink_pass {
            self.ink(1.4, C_INK)
        } else {
            self.pencil(1.6, C_SKETCH, 0.7)
        };
        self.closed(&iris, iris_brush, layer, stage, "iris", 0.4);

        let pupil = ellipse(
            eye.iris_center,
            eye.iris_rx * 0.45 * self.p.eyes.pupil,
            eye.iris_ry * 0.5 * self.p.eyes.pupil,
            14,
        );
        let pupil_brush = if ink_pass {
            self.ink(1.2, C_INK)
        } else {
            self.pencil(1.4, C_SKETCH, 0.8)
        };
        self.closed(&pupil, pupil_brush, layer, stage, "pupil", 0.25);

        // lash: a confident flick past the outer corner
        let dir = eye.outer.sub(eye.center).norm();
        let lash = vec![
            eye.upper[3],
            eye.outer,
            eye.outer.add(dir.mul(20.0)).add(Vec2::new(0.0, -7.0)),
        ];
        let lash_brush = if ink_pass {
            self.pen(5.5, C_INK)
        } else {
            self.pencil(3.4, C_SKETCH, 0.7)
        };
        self.curve(&lash, lash_brush, layer, stage, "eyelash", 0.3);

        // brows are inked once, in the eye stage; the lineart pass only
        // re-states the lids and iris.
        if !ink_pass {
            let (ex, ew, by) = (eye.center.x, self.geo.eye_w, self.geo.brow_y);
            let tilt = match self.p.expression {
                Expression::Serious => 12.0,
                Expression::Surprised => -10.0,
                _ => 0.0,
            };
            let raise = if matches!(self.p.expression, Expression::Smug) && side > 0.0 {
                10.0
            } else {
                0.0
            };
            let brow = vec![
                Vec2::new(ex - side * ew * 0.95, by + 10.0 + tilt),
                Vec2::new(ex - side * ew * 0.10, by - 8.0 - raise),
                Vec2::new(ex + side * ew * 0.88, by + 4.0 - raise * 0.4),
            ];
            self.curve(
                &brow,
                self.ink(3.0, C_INK),
                LAYER_LINEART,
                Stage::Eyes,
                "eyebrow",
                0.6,
            );
        }
    }

    // -- stage 6: nose and mouth -------------------------------------------

    fn stage_nose_mouth(&mut self) {
        let (cx, ny) = (self.geo.cx, self.geo.nose_y);
        let b = self.pencil(1.9, C_SKETCH, 0.7);
        self.curve(
            &[
                Vec2::new(cx + 9.0, ny - 26.0),
                Vec2::new(cx + 5.0, ny - 4.0),
                Vec2::new(cx - 5.0, ny + 6.0),
            ],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::NoseMouth,
            "nose bridge",
            0.9,
        );
        self.curve(
            &[Vec2::new(cx - 9.0, ny + 7.0), Vec2::new(cx - 2.0, ny + 9.0)],
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::NoseMouth,
            "nostril",
            0.4,
        );
        self.curve(
            &[Vec2::new(cx - 4.0, ny + 12.0), Vec2::new(cx + 4.0, ny + 11.0)],
            self.pencil(1.2, C_SKETCH, 0.45),
            LAYER_SHADOW,
            Stage::NoseMouth,
            "nose shadow",
            0.3,
        );

        match self.p.expression {
            Expression::Surprised => {
                let mouth = ellipse(Vec2::new(cx, self.geo.mouth_y + 4.0), 13.0, 17.0, 14);
                self.closed(
                    &mouth,
                    self.ink(2.2, C_INK),
                    LAYER_LINEART,
                    Stage::NoseMouth,
                    "open mouth",
                    0.4,
                );
            }
            _ => {
                let mouth = self.mouth_curve();
                self.curve(
                    &mouth,
                    self.ink(2.6, C_INK),
                    LAYER_LINEART,
                    Stage::NoseMouth,
                    "mouth",
                    0.5,
                );
                // lower lip hint
                let lip = vec![
                    Vec2::new(cx - 16.0, self.geo.mouth_y + 14.0),
                    Vec2::new(cx, self.geo.mouth_y + 19.0),
                    Vec2::new(cx + 16.0, self.geo.mouth_y + 14.0),
                ];
                self.curve(
                    &lip,
                    self.pencil(1.4, C_SKETCH, 0.5),
                    LAYER_CONSTRUCTION,
                    Stage::NoseMouth,
                    "lower lip",
                    0.5,
                );
            }
        }
    }

    fn mouth_curve(&self) -> Vec<Vec2> {
        let cx = self.geo.cx;
        let my = self.geo.mouth_y;
        match self.p.expression {
            Expression::Smile => vec![
                Vec2::new(cx - 33.0, my - 5.0),
                Vec2::new(cx, my + 11.0),
                Vec2::new(cx + 33.0, my - 5.0),
            ],
            Expression::Shy => vec![
                Vec2::new(cx - 19.0, my + 1.0),
                Vec2::new(cx - 6.0, my + 6.0),
                Vec2::new(cx + 4.0, my + 1.0),
                Vec2::new(cx + 17.0, my + 5.0),
            ],
            Expression::Serious => vec![
                Vec2::new(cx - 26.0, my + 3.0),
                Vec2::new(cx, my + 2.0),
                Vec2::new(cx + 26.0, my + 3.0),
            ],
            Expression::Smug => vec![
                Vec2::new(cx - 30.0, my + 5.0),
                Vec2::new(cx + 2.0, my + 2.0),
                Vec2::new(cx + 31.0, my - 9.0),
            ],
            _ => vec![
                Vec2::new(cx - 24.0, my),
                Vec2::new(cx, my + 5.0),
                Vec2::new(cx + 24.0, my),
            ],
        }
    }

    // -- stage 7: hair ------------------------------------------------------

    fn fringe_strands(&self) -> Vec<Vec<Vec2>> {
        let (cx, top, h, w) = (
            self.geo.cx,
            self.geo.top,
            self.geo.head_h,
            self.geo.half_w,
        );
        let part_x = cx + self.p.hair.parting * 70.0;
        // A fringe is drawn in clumps, not as combed lines: two or three
        // strands leave the parting apart and converge on one tapered tip,
        // with bare hair between the clumps. Evenly spaced parallel lines read
        // as a barcode.
        let clumps = [(0.06f32, 3usize), (0.31, 2), (0.60, 3), (0.87, 2)];
        let mut out = Vec::new();
        for (c, &(pos, count)) in clumps.iter().enumerate() {
            let target_x = cx + (pos * 2.0 - 1.0) * w * 0.98;
            let tip_y = self.geo.brow_y - 14.0 + ((c * 5 % 3) as f32 - 1.0) * 8.0;
            for k in 0..count {
                let spread = (k as f32 - (count as f32 - 1.0) * 0.5) * w * 0.05;
                let start = Vec2::new(
                    part_x + (target_x - part_x) * 0.05 + spread * 0.90,
                    top - 4.0 + k as f32 * 3.0,
                );
                let mid1 = Vec2::new(
                    part_x + (target_x - part_x) * 0.55 + spread * 0.75,
                    top + h * 0.05,
                );
                // the belly of the strand is pushed out, then the tip pulls
                // back in: that curve is what makes a fringe look drawn
                let mid2 = Vec2::new(
                    part_x + (target_x - part_x) * 0.97 + spread * 0.40,
                    self.geo.brow_y - 58.0,
                );
                let tip = Vec2::new(target_x + spread * 0.10, tip_y);
                out.push(vec![start, mid1, mid2, tip]);
            }
        }
        out
    }

    fn side_lock(&self, side: f32) -> Vec<Vec2> {
        let (cx, top, h, chin, w) = (
            self.geo.cx,
            self.geo.top,
            self.geo.head_h,
            self.geo.chin,
            self.geo.half_w,
        );
        let x = cx + side * w * 0.94;
        vec![
            Vec2::new(x, top + h * 0.18),
            Vec2::new(x + side * 24.0, self.geo.eye_y - 10.0),
            Vec2::new(x + side * 20.0, chin - 30.0),
            Vec2::new(x + side * 6.0, chin + 30.0),
        ]
    }

    /// Strands falling through one side mass, from the temple to the ends.
    ///
    /// They are placed relative to the silhouette width at each height, so
    /// they hug the outline of the hair instead of running over the cheek.
    fn side_strands(&self, side: f32) -> Vec<Vec<Vec2>> {
        let (cx, top, h, bottom) = (
            self.geo.cx,
            self.geo.top,
            self.geo.head_h,
            self.geo.hair_bottom,
        );
        let y_top0 = top + h * 0.20;
        let y_end0 = (bottom - 40.0).min(self.geo.shoulder_y - 24.0);
        let span = (y_end0 - y_top0).max(60.0);
        let at = |y: f32, frac: f32| {
            Vec2::new(
                cx + side * self.geo.hair_half_width_at(self.p, y) * frac,
                y,
            )
        };
        let mut out = Vec::new();
        for k in 0..4 {
            let frac = 0.62 + 0.11 * k as f32;
            let y0 = y_top0 + span * 0.03 * k as f32;
            let y1 = y0 + span * (0.72 + 0.08 * k as f32);
            out.push(vec![
                at(y0, frac),
                at(y0 + (y1 - y0) * 0.40, frac + 0.04),
                at(y0 + (y1 - y0) * 0.78, frac + 0.01),
                at(y1, frac - 0.03),
            ]);
        }
        out
    }

    /// Closed polygon for the hair highlight ("angel ring"): an arc that
    /// parallels the dome of the skull, thick at the crown and tapering away
    /// towards the temples, with a serrated lower edge. Anime hair is shaded
    /// as a ribbon of light, not as a stripe painted across the head.
    /// One glint of the angel ring, spanning the normalised arc range
    /// `t0..t1` (both in -1..=1). The band is an arch: high at the crown,
    /// drooping at the temples. Each glint tapers to zero at its own ends, so
    /// a row of glints reads as broken light rather than as a headband.
    fn highlight_band(&self, offset: f32, thickness: f32, t0: f32, t1: f32) -> Vec<Vec2> {
        let (cx, top, h, w) = (self.geo.cx, self.geo.top, self.geo.head_h, self.geo.half_w);
        let steps = 21;
        let mid_y = top + h * 0.075 + offset;
        let sag = h * 0.13;
        let mut upper = Vec::with_capacity(steps);
        let mut lower = Vec::with_capacity(steps);
        for i in 0..steps {
            let u = i as f32 / (steps - 1) as f32;
            let t = t0 + (t1 - t0) * u;
            let x = cx + t * w * 0.72;
            let y = mid_y + t * t * sag;
            // taper inside this glint: zero at both ends, so adjacent glints
            // are separated by real gaps of bare hair
            let local = u * 2.0 - 1.0;
            let taper = (1.0 - local * local).powf(0.55);
            // Fine, shallow serration. A regular sawtooth reads as a zipper and
            // a deep one as torn paper; 6% amplitude reads as light.
            let jitter = ((i * 7 + 3) % 5) as f32 / 4.0;
            let tooth = 0.94 + 0.06 * jitter;
            // A slow undulation keeps the lower edge from looking machined.
            let ripple = 1.0 + 0.07 * (i as f32 * 0.55).sin();
            let thick = h * 0.052 * taper * tooth * ripple * thickness;
            upper.push(Vec2::new(x, y));
            lower.push(Vec2::new(x, y + thick));
        }
        lower.reverse();
        upper.extend(lower);
        upper
    }

    fn stage_hair(&mut self) {
        let (cx, chin, w) = (self.geo.cx, self.geo.chin, self.geo.half_w);
        let sil = self.geo.hair_silhouette(self.p);
        self.closed(
            &sil,
            self.pencil(2.8, C_SKETCH, 0.7),
            LAYER_CONSTRUCTION,
            Stage::Hair,
            "hair silhouette",
            3.0,
        );

        // tails behind the shoulders
        match self.p.hair.style {
            HairStyle::TwinTails => {
                for side in [-1.0f32, 1.0] {
                    self.draw_tail(side, 1.28, 300.0);
                }
            }
            HairStyle::Ponytail => self.draw_tail(1.0, 1.30, 360.0),
            _ => {}
        }

        // fringe strands radiating from the parting
        let strands = self.fringe_strands();
        for (i, strand) in strands.iter().enumerate() {
            let brush = if i % 3 == 0 {
                self.pen(4.2, C_SKETCH).with_opacity(0.65)
            } else {
                self.pencil(2.2, C_SKETCH, 0.65)
            };
            self.curve(
                strand,
                brush,
                LAYER_CONSTRUCTION,
                Stage::Hair,
                "fringe strand",
                1.6,
            );
        }

        // side locks framing the face
        for side in [-1.0f32, 1.0] {
            let lock = self.side_lock(side);
            self.curve(
                &lock,
                self.pen(5.0, C_SKETCH).with_opacity(0.55),
                LAYER_CONSTRUCTION,
                Stage::Hair,
                "side lock",
                1.4,
            );
        }

        // interior strands give the mass direction; they live below the jaw so
        // they never cross the face
        if self.geo.hair_bottom > chin + 150.0 {
            for i in 0..6 {
                let t = i as f32 / 5.0;
                let x = cx + (t * 2.0 - 1.0) * w * 1.02;
                let strand = vec![
                    Vec2::new(x, chin + 40.0),
                    Vec2::new(x * 1.03, chin + 140.0),
                    Vec2::new(x * 1.04, self.geo.hair_bottom - 70.0),
                    Vec2::new(x * 1.01, self.geo.hair_bottom - 20.0),
                ];
                self.curve(
                    &strand,
                    self.pencil(1.5, C_SKETCH, 0.4),
                    LAYER_CONSTRUCTION,
                    Stage::Hair,
                    "hair strand",
                    2.2,
                );
            }
        }
    }

    fn tail_polygon(&self, side: f32, offset: f32, length: f32) -> Vec<Vec2> {
        let (cx, top, h) = (self.geo.cx, self.geo.top, self.geo.head_h);
        let x = cx + side * self.geo.half_w * offset;
        let y0 = top + h * 0.42;
        vec![
            Vec2::new(x - side * 18.0, y0),
            Vec2::new(x + side * 52.0, y0 + 46.0),
            Vec2::new(x + side * 68.0, y0 + length * 0.55),
            Vec2::new(x + side * 36.0, y0 + length),
            Vec2::new(x - side * 8.0, y0 + length - 34.0),
            Vec2::new(x - side * 22.0, y0 + length * 0.35),
        ]
    }

    fn draw_tail(&mut self, side: f32, offset: f32, length: f32) {
        let pts = self.tail_polygon(side, offset, length);
        self.closed(
            &pts,
            self.pencil(2.6, C_SKETCH, 0.65),
            LAYER_CONSTRUCTION,
            Stage::Hair,
            "hair tail",
            2.4,
        );
    }

    // -- stage 8: body ------------------------------------------------------

    fn stage_body(&mut self) {
        let (cx, sh) = (self.geo.cx, self.geo.shoulder_y);
        let b = self.pencil(2.4, C_SKETCH, 0.62);
        let right = self.geo.body_outline_right();
        self.curve(
            &right,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Body,
            "right torso",
            1.8,
        );
        let left = mirror_x(&right, cx);
        self.curve(
            &left,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Body,
            "left torso",
            1.8,
        );

        // collarbones
        for side in [-1.0f32, 1.0] {
            self.curve(
                &[
                    Vec2::new(cx + side * 24.0, sh - 46.0),
                    Vec2::new(cx + side * 90.0, sh - 34.0),
                    Vec2::new(cx + side * 150.0, sh - 18.0),
                ],
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Body,
                "collarbone",
                1.0,
            );
        }
    }

    // -- stage 9: clothing --------------------------------------------------

    fn stage_clothing(&mut self) {
        let (cx, sh) = (self.geo.cx, self.geo.shoulder_y);
        let b = self.pen(3.0, C_SKETCH).with_opacity(0.7);

        if matches!(self.p.outfit.kind, OutfitKind::Hoodie) {

            for side in [-1.0f32, 1.0] {
                self.curve(
                    &[
                        Vec2::new(cx + side * 30.0, sh + 54.0),
                        Vec2::new(cx + side * 36.0, sh + 132.0),
                    ],
                    b.clone(),
                    LAYER_CONSTRUCTION,
                    Stage::Clothing,
                    "drawstring",
                    0.6,
                );
            }
            self.curve(
                &[
                    Vec2::new(cx - 92.0, sh + 300.0),
                    Vec2::new(cx, sh + 316.0),
                    Vec2::new(cx + 92.0, sh + 300.0),
                ],
                b.clone(),
                LAYER_CONSTRUCTION,
                Stage::Clothing,
                "pocket",
                1.0,
            );
            return;
        }

        let collar = vec![
            Vec2::new(cx - 112.0, sh - 8.0),
            Vec2::new(cx - 66.0, sh + 54.0),
            Vec2::new(cx, sh + 98.0),
            Vec2::new(cx + 66.0, sh + 54.0),
            Vec2::new(cx + 112.0, sh - 8.0),
        ];
        self.curve(
            &collar,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Clothing,
            "sailor collar",
            1.0,
        );
        let band = vec![
            Vec2::new(cx - 104.0, sh + 8.0),
            Vec2::new(cx - 62.0, sh + 62.0),
            Vec2::new(cx, sh + 104.0),
            Vec2::new(cx + 62.0, sh + 62.0),
            Vec2::new(cx + 104.0, sh + 8.0),
        ];
        self.curve(
            &band,
            b.clone(),
            LAYER_CONSTRUCTION,
            Stage::Clothing,
            "collar band",
            0.9,
        );

        // ribbon: two triangular loops and a knot
        let knot = Vec2::new(cx, sh + 118.0);
        for side in [-1.0f32, 1.0] {
            let bow = vec![
                knot,
                Vec2::new(cx + side * 36.0, sh + 100.0),
                Vec2::new(cx + side * 42.0, sh + 140.0),
            ];
            self.closed(
                &bow,
                self.ink(2.4, C_INK),
                LAYER_LINEART,
                Stage::Clothing,
                "ribbon",
                0.4,
            );
        }
        let knot_ring = ellipse(knot, 9.0, 9.0, 12);
        self.closed(
            &knot_ring,
            self.ink(2.2, C_INK),
            LAYER_LINEART,
            Stage::Clothing,
            "ribbon knot",
            0.3,
        );

        // sleeve seams read as arms rather than a slab
        for side in [-1.0f32, 1.0] {
            self.curve(
                &[
                    Vec2::new(cx + side * self.geo.shoulder_half * 0.90, sh + 34.0),
                    Vec2::new(cx + side * self.geo.shoulder_half * 0.97, sh + 190.0),
                    Vec2::new(cx + side * self.geo.shoulder_half * 0.94, sh + 360.0),
                ],
                self.pencil(2.0, C_SKETCH, 0.5),
                LAYER_CONSTRUCTION,
                Stage::Clothing,
                "sleeve seam",
                1.2,
            );
        }

        // fabric folds
        for i in 0..3 {
            let x = cx - 90.0 + i as f32 * 90.0;
            self.curve(
                &[
                    Vec2::new(x, sh + 190.0),
                    Vec2::new(x + 12.0, sh + 250.0),
                    Vec2::new(x + 4.0, sh + 320.0),
                ],
                self.pencil(1.6, C_SKETCH, 0.45),
                LAYER_CONSTRUCTION,
                Stage::Clothing,
                "fabric fold",
                1.4,
            );
        }
    }

    // -- stage 10: clean lineart -------------------------------------------

    fn stage_lineart(&mut self) {
        let ink = self.ink(3.6, C_INK);
        let right = self.geo.face_right();
        let right_ink = right[2..].to_vec();
        self.curve(
            &right_ink,
            ink.clone(),
            LAYER_LINEART,
            Stage::Lineart,
            "face contour (ink)",
            0.55,
        );
        let left_ink = mirror_x(&right_ink, self.geo.cx);
        self.curve(
            &left_ink,
            ink.clone(),
            LAYER_LINEART,
            Stage::Lineart,
            "face contour (ink)",
            0.55,
        );

        // eyes get a second, confident pass
        for side in [-1.0f32, 1.0] {
            self.draw_eye(side, true);
        }

        // tails need their own contour, they are outside the main mass
        match self.p.hair.style {
            HairStyle::TwinTails => {
                for side in [-1.0f32, 1.0] {
                    let tail = self.tail_polygon(side, 1.20, 470.0);
                    self.closed(
                        &tail,
                        self.pen(3.4, C_INK),
                        LAYER_LINEART,
                        Stage::Lineart,
                        "tail outline (ink)",
                        0.5,
                    );
                }
            }
            HairStyle::Ponytail => {
                let tail = self.tail_polygon(1.0, 1.22, 520.0);
                self.closed(
                    &tail,
                    self.pen(3.4, C_INK),
                    LAYER_LINEART,
                    Stage::Lineart,
                    "ponytail outline (ink)",
                    0.5,
                );
            }
            _ => {}
        }

        // Hair contour. It stops at the shoulder line: the mass continues
        // behind the body, and outlining it there would draw a line across the
        // chest.
        let sil = self.geo.hair_silhouette(self.p);
        let cut = self.geo.shoulder_y - 6.0;
        let mid = sil.len() / 2;
        let right: Vec<Vec2> = sil[..=mid].iter().copied().filter(|p| p.y < cut).collect();
        let left: Vec<Vec2> = sil[mid..].iter().copied().filter(|p| p.y < cut).collect();
        if right.len() > 2 {
            self.curve(
                &right,
                self.pen(3.4, C_INK),
                LAYER_LINEART,
                Stage::Lineart,
                "hair contour (ink)",
                0.5,
            );
        }
        if left.len() > 2 {
            self.curve(
                &left,
                self.pen(3.4, C_INK),
                LAYER_LINEART,
                Stage::Lineart,
                "hair contour (ink)",
                0.5,
            );
        }

        // side locks
        for side in [-1.0f32, 1.0] {
            let x = self.geo.cx + side * self.geo.half_w * 0.94;
            self.curve(
                &[
                    Vec2::new(x, self.geo.top + self.geo.head_h * 0.18),
                    Vec2::new(x + side * 24.0, self.geo.eye_y - 10.0),
                    Vec2::new(x + side * 20.0, self.geo.chin - 30.0),
                ],
                self.pen(4.0, C_INK),
                LAYER_LINEART,
                Stage::Lineart,
                "side lock (ink)",
                0.6,
            );
        }

        // fringe: the final image hides the construction layer, so the hair
        // strands have to be re-stated in ink
        for (i, strand) in self.fringe_strands().iter().enumerate() {
            let brush = if i % 3 == 0 {
                self.pen(3.4, C_INK)
            } else {
                self.ink(2.2, C_INK)
            };
            self.curve(strand, brush, LAYER_LINEART, Stage::Lineart, "fringe (ink)", 0.45);
        }

        // Strands across the side masses. Without them a long hairstyle is one
        // flat slab of colour; with them the mass reads as hair falling in
        // locks. They follow the silhouette, so they curve out at the cheek
        // and drift in again at the ends.
        for side in [-1.0f32, 1.0] {
            for (k, strand) in self.side_strands(side).iter().enumerate() {
                let brush = if k % 2 == 0 {
                    self.ink(2.4, C_INK).with_opacity(0.34)
                } else {
                    self.ink(1.7, C_INK).with_opacity(0.24)
                };
                self.curve(strand, brush, LAYER_LINEART, Stage::Lineart, "side strand (ink)", 0.5);
            }
        }

        // nose
        let (nx, ny) = (self.geo.cx, self.geo.nose_y);
        self.curve(
            &[
                Vec2::new(nx + 7.0, ny - 22.0),
                Vec2::new(nx + 4.0, ny - 4.0),
                Vec2::new(nx - 5.0, ny + 5.0),
            ],
            self.ink(2.2, C_INK),
            LAYER_LINEART,
            Stage::Lineart,
            "nose (ink)",
            0.3,
        );

        // hoodie details in ink
        if matches!(self.p.outfit.kind, OutfitKind::Hoodie) {
            let (cx, sh) = (self.geo.cx, self.geo.shoulder_y);

            for side in [-1.0f32, 1.0] {
                self.curve(
                    &[
                        Vec2::new(cx + side * 30.0, sh + 54.0),
                        Vec2::new(cx + side * 36.0, sh + 132.0),
                    ],
                    self.ink(2.4, C_INK),
                    LAYER_LINEART,
                    Stage::Lineart,
                    "drawstring (ink)",
                    0.4,
                );
            }
        }

        // shirt silhouette: smooth shoulders, sharp V neckline. A spline over
        // the whole polygon would round the neckline into a bib.
        let (cx, sh) = (self.geo.cx, self.geo.shoulder_y);
        let half = self.geo.shoulder_half;
        for side in [-1.0f32, 1.0] {
            let side_pts = vec![
                Vec2::new(cx + side * 78.0, sh - 16.0),
                Vec2::new(cx + side * half * 0.45, sh - 10.0),
                Vec2::new(cx + side * half * 0.90, sh + 10.0),
                Vec2::new(cx + side * half, sh + 70.0),
                Vec2::new(cx + side * half * 1.03, sh + 210.0),
                Vec2::new(cx + side * half * 1.06, self.geo.bottom),
            ];
            self.curve(
                &side_pts,
                self.pen(3.4, C_INK),
                LAYER_LINEART,
                Stage::Lineart,
                "shirt side (ink)",
                0.4,
            );
        }
        let neckline = vec![
            Vec2::new(cx - 78.0, sh - 16.0),
            Vec2::new(cx, sh + 64.0),
            Vec2::new(cx + 78.0, sh - 16.0),
        ];
        self.doc.polyline(
            &neckline,
            self.pen(3.4, C_INK),
            LAYER_LINEART,
            Stage::Lineart,
            "neckline (ink)",
        );
        if !matches!(self.p.outfit.kind, OutfitKind::Hoodie) {
            let band = vec![
                Vec2::new(cx - 104.0, sh + 8.0),
                Vec2::new(cx - 62.0, sh + 62.0),
                Vec2::new(cx, sh + 104.0),
                Vec2::new(cx + 62.0, sh + 62.0),
                Vec2::new(cx + 104.0, sh + 8.0),
            ];
            self.curve(
                &band,
                self.ink(2.6, C_INK),
                LAYER_LINEART,
                Stage::Lineart,
                "collar band (ink)",
                0.5,
            );
        }

        // neck and shoulders
        let (cx, chin, sh) = (self.geo.cx, self.geo.chin, self.geo.shoulder_y);
        for side in [-1.0f32, 1.0] {
            self.curve(
                &[
                    Vec2::new(cx + side * self.geo.half_w * 0.34, chin - 4.0),
                    Vec2::new(cx + side * self.geo.half_w * 0.40, sh - 70.0),
                    Vec2::new(cx + side * self.geo.half_w * 0.70, sh - 26.0),
                    Vec2::new(cx + side * self.geo.shoulder_half * 0.86, sh + 6.0),
                ],
                ink.clone(),
                LAYER_LINEART,
                Stage::Lineart,
                "neck & shoulder (ink)",
                0.7,
            );
        }
    }

    // -- stage 11: flat colour ---------------------------------------------

    fn stage_color(&mut self) {
        let (cx, top, h) = (self.geo.cx, self.geo.top, self.geo.head_h);
        // atmospheric wash behind the character
        let halo = self.air(760.0, Rgba::rgb(198, 214, 238), 0.42);
        self.curve(
            &[Vec2::new(cx, top + h * 0.62)],
            halo,
            LAYER_BACKGROUND,
            Stage::Color,
            "background wash",
            0.0,
        );

        let hair = self.p.hair.color;

        // hair mass first, then the face covers it
        let silhouette = self.geo.hair_silhouette(self.p);
        self.fill(
            silhouette,
            hair,
            LAYER_FLAT,
            Stage::Color,
            "hair base",
        );

        // neck behind the face
        let neck = self.geo.neck_polygon();
        self.fill(neck, C_SKIN, LAYER_FLAT, Stage::Color, "neck base");
        for side in [-1.0f32, 1.0] {
            let e = self.geo.ear_center(side);
            let ear = ellipse(e, 24.0, 36.0, 16);
            self.fill(ear, C_SKIN, LAYER_FLAT, Stage::Color, "ear base");
        }

        let face = self.geo.face_polygon();
        self.fill(face, C_SKIN, LAYER_FLAT, Stage::Color, "face base");

        // fringe sits in front of the forehead
        let bangs = self.geo.bangs_polygon(self.p);
        self.fill(bangs, hair, LAYER_FLAT, Stage::Color, "fringe base");

        // chest behind the collar opening, otherwise the V shows background
        let sh0 = self.geo.shoulder_y;
        let nw = self.geo.half_w * 0.37;
        let chest = vec![
            Vec2::new(cx - nw * 2.2, sh0 - 18.0),
            Vec2::new(cx - 86.0, sh0 + 88.0),
            Vec2::new(cx, sh0 + 114.0),
            Vec2::new(cx + 86.0, sh0 + 88.0),
            Vec2::new(cx + nw * 2.2, sh0 - 18.0),
        ];
        self.fill(chest, C_SKIN, LAYER_FLAT, Stage::Color, "chest base");

        let outfit = self.geo.outfit_polygon();
        self.fill(
            outfit,
            self.p.outfit.color,
            LAYER_FLAT,
            Stage::Color,
            "outfit base",
        );

        // tails hang over the shoulders, so they are filled after the shirt
        match self.p.hair.style {
            HairStyle::TwinTails => {
                for side in [-1.0f32, 1.0] {
                    let tail = self.tail_polygon(side, 1.20, 470.0);
                    self.fill(tail, hair, LAYER_FLAT, Stage::Color, "twin tail base");
                }
            }
            HairStyle::Ponytail => {
                let tail = self.tail_polygon(1.0, 1.22, 520.0);
                self.fill(tail, hair, LAYER_FLAT, Stage::Color, "ponytail base");
            }
            _ => {}
        }

        // ribbon (uniform only)
        let sh = self.geo.shoulder_y;
        let knot = Vec2::new(cx, sh + 118.0);
        for side in if matches!(self.p.outfit.kind, OutfitKind::Hoodie) {
            Vec::<f32>::new()
        } else {
            vec![-1.0f32, 1.0]
        } {
            let bow = vec![
                knot,
                Vec2::new(cx + side * 36.0, sh + 100.0),
                Vec2::new(cx + side * 42.0, sh + 140.0),
            ];
            self.fill(
                bow,
                self.p.outfit.accent,
                LAYER_FLAT,
                Stage::Color,
                "ribbon fill",
            );
        }
        let knot_fill = ellipse(knot, 9.0, 9.0, 12);
        self.fill(
            knot_fill,
            self.p.outfit.accent,
            LAYER_FLAT,
            Stage::Color,
            "ribbon knot fill",
        );

        // eyes
        for side in [-1.0f32, 1.0] {
            let eye = self.geo.eye(side, self.p);
            let clip = eye.polygon.clone();
            let sclera = ellipse(eye.center, self.geo.eye_w, self.geo.eye_h * 1.1, 20);
            self.fill_clipped(
                sclera,
                C_SCLERA,
                LAYER_FLAT,
                Stage::Color,
                "sclera",
                clip.clone(),
            );
            let iris = ellipse(eye.iris_center, eye.iris_rx, eye.iris_ry, 22);
            self.fill_clipped(
                iris.clone(),
                self.p.eyes.color,
                LAYER_FLAT,
                Stage::Color,
                "iris",
                clip.clone(),
            );
            // A flat disc of colour reads as a sticker. Anime irises are
            // darker under the lid and lit from below, so the two crescents
            // are what give the eye depth.
            let shade = ellipse(
                Vec2::new(eye.iris_center.x, eye.iris_center.y - eye.iris_ry * 0.66),
                eye.iris_rx * 0.97,
                eye.iris_ry * 0.64,
                20,
            );
            self.fill_clipped(
                shade,
                // Shadow = absorption, not added light: keep the encoded lerp
                // toward near-black until multiplicative albedo shading lands.
                self.p.eyes.color.mix(Rgba::rgb(26, 20, 38), 0.46),
                LAYER_FLAT,
                Stage::Color,
                "iris shade",
                clip.clone(),
            );
            let glow = ellipse(
                Vec2::new(eye.iris_center.x, eye.iris_center.y + eye.iris_ry * 0.58),
                eye.iris_rx * 0.74,
                eye.iris_ry * 0.32,
                18,
            );
            self.fill_clipped(
                glow,
                // Glow is added light, so mix in linear radiance.
                self.p.eyes.color.mix_linear(Rgba::rgb(255, 255, 255), 0.52),
                LAYER_FLAT,
                Stage::Color,
                "iris glow",
                clip.clone(),
            );
            let pupil = ellipse(
                eye.iris_center,
                eye.iris_rx * 0.45 * self.p.eyes.pupil,
                eye.iris_ry * 0.5 * self.p.eyes.pupil,
                16,
            );
            self.fill_clipped(
                pupil,
                C_PUPIL,
                LAYER_FLAT,
                Stage::Color,
                "pupil",
                clip,
            );
        }

        // lips
        if !matches!(self.p.expression, Expression::Surprised) {
            let lips = ellipse(Vec2::new(cx, self.geo.mouth_y + 8.0), 22.0, 9.0, 14);
            self.fill(
                lips,
                C_LIP.with_alpha(0.55),
                LAYER_FLAT,
                Stage::Color,
                "lip colour",
            );
        }

        // hair highlight: the "angel ring". It follows the skull, tapers to
        // nothing at both temples and has a serrated lower edge, because that
        // is the shape anime hair actually uses. A blurred stroke reads as a
        // smudge and a thin line as a scribble; the band is what makes a mass
        // of hair look glossy.
        let lum = hair.luminance();
        // A highlight is *added light*, so the mix happens in linear radiance.
        // Three bands, because one threshold is not enough:
        //   light hair  — little headroom, go almost to white;
        //   mid-tone    — a 42% white mix brightens while keeping the hue
        //                 (pink hair must not turn blue-grey);
        //   dark hair   — a neutral band reads as dust, so use a cool tint
        //                 like the rim light on black hair in anime.
        let hi_color = if lum > 0.72 {
            hair.mix_linear(Rgba::rgb(255, 255, 255), 0.80)
        } else if lum > 0.42 {
            hair.mix_linear(Rgba::rgb(255, 255, 255), 0.42)
        } else {
            hair.mix_linear(Rgba::rgb(198, 210, 246), 0.58)
        };
        let hair_clip = self.geo.hair_silhouette(self.p);
        // Broken into three glints: a single continuous strip reads as a
        // headband, while gaps of bare hair read as reflected light.
        let thick = if lum > 0.72 { 1.0 } else { 0.82 };
        for (t0, t1) in [(-0.95f32, -0.42f32), (-0.28, 0.28), (0.42, 0.95)] {
            let band = self.highlight_band(0.0, thick, t0, t1);
            self.fill_clipped(
                band,
                hi_color,
                LAYER_SHADOW,
                Stage::Color,
                "hair highlight band",
                hair_clip.clone(),
            );
        }

        // eye catchlights
        for side in [-1.0f32, 1.0] {
            let eye = self.geo.eye(side, self.p);
            let hi = Vec2::new(
                eye.iris_center.x - side * eye.iris_rx * 0.42,
                eye.iris_center.y - eye.iris_ry * 0.38,
            );
            let dot = ellipse(hi, 8.5, 9.5, 12);
            self.fill_clipped(
                dot,
                Rgba::rgb(255, 255, 255),
                LAYER_FX,
                Stage::Color,
                "eye catchlight",
                eye.polygon.clone(),
            );
        }
    }

    // -- stage 12: shading --------------------------------------------------

    fn stage_shading(&mut self) {
        let (cx, chin, h) = (self.geo.cx, self.geo.chin, self.geo.head_h);

        // Shadow under the chin. A row of parallel ticks reads as a barcode,
        // so the shadow is an airbrushed mass with three short hatches at one
        // side only, where a hand would actually put them.
        let neck_shadow = self.air(58.0, C_SKIN_SHADOW, 0.34);
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let x = cx + (t * 2.0 - 1.0) * 46.0;
            self.curve(
                &[Vec2::new(x, chin + 20.0)],
                neck_shadow.clone(),
                LAYER_SHADOW,
                Stage::Shading,
                "neck shadow",
                0.0,
            );
        }
        for i in 0..3 {
            let x = cx + 16.0 + i as f32 * 13.0;
            self.curve(
                &[
                    Vec2::new(x, chin + 14.0),
                    Vec2::new(x + 5.0, chin + 30.0),
                ],
                self.pencil(1.5, C_SKIN_SHADOW, 0.35),
                LAYER_SHADOW,
                Stage::Shading,
                "neck hatch",
                0.5,
            );
        }

        // Hair shadow on the forehead: a soft band under the fringe tips. The
        // previous row of vertical ticks looked like comb teeth.
        let fore = self.air(44.0, Rgba::rgb(214, 190, 196), 0.30);
        for i in 0..7 {
            let t = i as f32 / 6.0;
            let x = cx + (t * 2.0 - 1.0) * self.geo.half_w * 0.74;
            let y = self.geo.brow_y - 26.0 + (t * 2.0 - 1.0).abs() * 10.0;
            self.curve(
                &[Vec2::new(x, y)],
                fore.clone(),
                LAYER_SHADOW,
                Stage::Shading,
                "forehead shadow",
                0.0,
            );
        }

        // blush
        for side in [-1.0f32, 1.0] {
            let c = Vec2::new(cx + side * self.geo.half_w * 0.52, self.geo.eye_y + 62.0);
            self.curve(
                &[c],
                self.air(46.0, C_BLUSH, 0.16),
                LAYER_SHADOW,
                Stage::Shading,
                "blush",
                0.0,
            );
            for i in 0..2 {
                let x = c.x - 10.0 + i as f32 * 20.0;
                self.curve(
                    &[Vec2::new(x, c.y - 6.0), Vec2::new(x + 6.0, c.y + 6.0)],
                    self.pencil(1.1, C_BLUSH, 0.35),
                    LAYER_SHADOW,
                    Stage::Shading,
                    "blush line",
                    0.4,
                );
            }
        }

        // eyelid shadow
        for side in [-1.0f32, 1.0] {
            let eye = self.geo.eye(side, self.p);
            let y = eye.center.y - self.geo.eye_h * 1.1;
            self.curve(
                &[
                    Vec2::new(eye.center.x - self.geo.eye_w * 0.7, y + 4.0),
                    Vec2::new(eye.center.x, y - 3.0),
                    Vec2::new(eye.center.x + self.geo.eye_w * 0.7, y + 4.0),
                ],
                self.pencil(1.4, Rgba::rgb(206, 186, 196), 0.45),
                LAYER_SHADOW,
                Stage::Shading,
                "eyelid shadow",
                0.5,
            );
        }

        // hair reads as a mass only if its inner edge is shaded.
        // Shadow = absorption, so keep the encoded lerp toward near-black for
        // now; linear-light multiplication belongs to the shading pass.
        let hair_shadow = self.p.hair.color.mix(Rgba::rgb(40, 36, 54), 0.45);
        for side in [-1.0f32, 1.0] {
            let x = cx + side * self.geo.half_w * 0.90;
            self.curve(
                &[
                    Vec2::new(x, self.geo.top + h * 0.26),
                    Vec2::new(x + side * 10.0, self.geo.eye_y),
                    Vec2::new(x - side * 6.0, self.geo.chin - 30.0),
                ],
                self.air(62.0, hair_shadow, 0.13),
                LAYER_SHADOW,
                Stage::Shading,
                "hair inner shadow",
                0.0,
            );
        }

        // soft shadow just under the fringe tips (kept on the forehead, not on
        // the hair mass)
        for i in 0..3 {
            let x = cx + (i as f32 - 1.0) * self.geo.half_w * 0.42;
            self.curve(
                &[Vec2::new(x, self.geo.brow_y - 6.0)],
                self.air(52.0, Rgba::rgb(226, 206, 208), 0.10),
                LAYER_SHADOW,
                Stage::Shading,
                "fringe shadow",
                0.0,
            );
        }
    }
}

/// Second pass after evaluation: redraw the contour where the drawn line drifted
/// from the plan. This is a real correction, not a decoration: it is only added
/// when the evaluator measured an error above tolerance.
pub fn apply_corrections(doc: &mut Document, plan: &DrawingPlan, issues: &[Issue]) -> usize {
    let needs_contour = issues
        .iter()
        .any(|i| i.kind == "symmetry" || i.kind == "proportion" || i.kind == "chin");
    if !needs_contour {
        return 0;
    }
    let brush = Brush::new(BrushKind::Ink, 3.8, C_INK).with_jitter(0.05);
    let mut added = 0;
    let right: Vec<Vec2> = plan.outline[3..].to_vec();
    let left = mirror_x(&right, plan.chin.x);
    for (pts, role) in [
        (right, "correction: right contour"),
        (left, "correction: left contour"),
    ] {
        let id = doc.actions.len() as u32 + 1;
        let stroke = Stroke::from_control_points(
            &pts,
            brush.clone(),
            LAYER_LINEART,
            Stage::Correction,
            role,
            doc.meta.seed ^ id as u64,
        );
        doc.push(Action::Stroke(stroke));
        added += 1;
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CharacterProfile {
        CharacterProfile::from_prompt("silver hair, red eyes, shy smile", 11)
    }

    #[test]
    fn draws_every_stage() {
        let art = draw(&profile());
        let stages: Vec<Stage> = art.document.actions.iter().map(|a| a.stage()).collect();
        for s in [
            Stage::Composition,
            Stage::Rough,
            Stage::Construction,
            Stage::Face,
            Stage::Eyes,
            Stage::NoseMouth,
            Stage::Hair,
            Stage::Body,
            Stage::Clothing,
            Stage::Lineart,
            Stage::Color,
            Stage::Shading,
        ] {
            assert!(stages.contains(&s), "missing stage {:?}", s);
        }
    }

    #[test]
    fn is_deterministic() {
        let a = draw(&profile()).document.to_json();
        let b = draw(&profile()).document.to_json();
        assert_eq!(a, b);
    }

    #[test]
    fn strokes_stay_on_canvas() {
        let art = draw(&profile());
        let b = art.document.bounds();
        assert!(b.min.x > -80.0 && b.max.x < 1280.0);
        assert!(b.min.y > -80.0 && b.max.y < 1680.0);
    }

    #[test]
    fn has_meaningful_volume() {
        let art = draw(&profile());
        assert!(art.document.stroke_count() > 80);
        assert!(art.document.fill_count() >= 8);
        assert!(art.document.point_count() > 3000);
    }

    #[test]
    fn twin_tails_add_geometry() {
        let a = draw(&CharacterProfile::from_prompt("twin tails, pink hair", 4));
        let b = draw(&CharacterProfile::from_prompt("short bob, pink hair", 4));
        assert!(a.document.stroke_count() > b.document.stroke_count());
    }
}
