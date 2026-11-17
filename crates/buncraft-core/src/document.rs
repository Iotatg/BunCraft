//! The document: layers, drawing actions, timeline and self-evaluation.

use serde::{Deserialize, Serialize};

use crate::geom::{Bounds, Vec2};
use crate::profile::CharacterProfile;
use crate::stroke::{BrushKind, Rgba, Stage, Stroke};

pub const LAYER_BACKGROUND: u8 = 0;
pub const LAYER_CONSTRUCTION: u8 = 1;
pub const LAYER_FLAT: u8 = 2;
pub const LAYER_SHADOW: u8 = 3;
pub const LAYER_LINEART: u8 = 4;
pub const LAYER_FX: u8 = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub background: Rgba,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            background: Rgba::rgb(252, 250, 246),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: u8,
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
}

impl Layer {
    pub fn new(id: u8, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            opacity: 1.0,
            visible: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FillRegion {
    pub id: u32,
    pub layer: u8,
    pub stage: Stage,
    pub polygon: Vec<Vec2>,
    pub color: Rgba,
    pub opacity: f32,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip: Option<Vec<Vec2>>,
}

impl FillRegion {
    pub fn new(
        id: u32,
        layer: u8,
        stage: Stage,
        polygon: Vec<Vec2>,
        color: Rgba,
        role: impl Into<String>,
    ) -> Self {
        Self {
            id,
            layer,
            stage,
            polygon,
            color,
            opacity: 1.0,
            role: role.into(),
            clip: None,
        }
    }

    pub fn bounds(&self) -> Bounds {
        let mut b = Bounds::empty();
        for p in &self.polygon {
            b.include(*p);
        }
        b
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Stroke(Stroke),
    Fill(FillRegion),
}

impl Action {
    pub fn layer(&self) -> u8 {
        match self {
            Action::Stroke(s) => s.layer,
            Action::Fill(f) => f.layer,
        }
    }

    pub fn stage(&self) -> Stage {
        match self {
            Action::Stroke(s) => s.stage,
            Action::Fill(f) => f.stage,
        }
    }

    pub fn role(&self) -> &str {
        match self {
            Action::Stroke(s) => &s.role,
            Action::Fill(f) => &f.role,
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Action::Stroke(s) => s.bounds(),
            Action::Fill(f) => f.bounds(),
        }
    }

    pub fn length(&self) -> f32 {
        match self {
            Action::Stroke(s) => s.length(),
            Action::Fill(_) => 0.0,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Action::Stroke(s) => s.id,
            Action::Fill(f) => f.id,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Timing {
    pub start_ms: f64,
    pub duration_ms: f64,
}

impl Timing {
    pub fn end_ms(&self) -> f64 {
        self.start_ms + self.duration_ms
    }

    /// 0..1 progress at time `t`.
    pub fn progress(&self, t: f64) -> f32 {
        if t <= self.start_ms {
            0.0
        } else if t >= self.end_ms() {
            1.0
        } else {
            ((t - self.start_ms) / self.duration_ms.max(0.001)) as f32
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocMeta {
    pub title: String,
    pub prompt: String,
    pub seed: u64,
    pub created_at: String,
    pub generator: String,
    pub engine_version: String,
}

impl DocMeta {
    pub fn new(title: impl Into<String>, prompt: impl Into<String>, seed: u64) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            seed,
            created_at: now_iso8601(),
            generator: "buncraft".to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Minimal UTC timestamp without pulling in a date crate.
fn now_iso8601() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // days since epoch -> civil date (Howard Hinnant's algorithm)
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Issue {
    pub kind: String,
    pub detail: String,
    /// 0 = cosmetic, 1 = the drawing is visibly wrong.
    pub severity: f32,
    pub corrected: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Evaluation {
    /// Mean left/right landmark deviation in pixels, after correction.
    pub symmetry_error: f32,
    /// Relative error of measured proportions against the plan, after correction.
    pub proportion_error: f32,
    /// Measurement before the correction pass (0 when no correction ran).
    #[serde(default)]
    pub symmetry_error_before: f32,
    #[serde(default)]
    pub proportion_error_before: f32,
    /// Fraction of the canvas touched by ink.
    pub coverage: f32,
    pub strokes: usize,
    pub points: usize,
    pub duration_ms: f64,
    pub issues: Vec<Issue>,
    pub corrections: usize,
    /// Geometric consistency score, 0..100. Stored (not only computed) so it
    /// travels with the JSON report and the replay player.
    pub score: f32,
}

impl Evaluation {
    /// Single 0..100 quality score, weighted towards what a viewer notices.
    pub fn compute_score(&self) -> f32 {
        let sym = (1.0 - (self.symmetry_error / 12.0)).clamp(0.0, 1.0);
        let prop = (1.0 - (self.proportion_error / 0.18)).clamp(0.0, 1.0);
        (sym * 0.45 + prop * 0.55) * 100.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub meta: DocMeta,
    pub canvas: Canvas,
    pub layers: Vec<Layer>,
    pub actions: Vec<Action>,
    pub timings: Vec<Timing>,
    pub duration_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<CharacterProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<Evaluation>,
}

impl Document {
    pub fn new(canvas: Canvas, meta: DocMeta) -> Self {
        let layers = vec![
            Layer::new(LAYER_BACKGROUND, "background"),
            Layer::new(LAYER_CONSTRUCTION, "construction"),
            Layer::new(LAYER_FLAT, "flat colour"),
            Layer::new(LAYER_SHADOW, "shadow"),
            Layer::new(LAYER_LINEART, "lineart"),
            Layer::new(LAYER_FX, "effects"),
        ];
        Self {
            meta,
            canvas,
            layers,
            actions: Vec::new(),
            timings: Vec::new(),
            duration_ms: 0.0,
            profile: None,
            evaluation: None,
        }
    }

    pub fn push(&mut self, mut action: Action) -> u32 {
        let id = self.actions.len() as u32 + 1;
        match &mut action {
            Action::Stroke(s) => s.id = id,
            Action::Fill(f) => f.id = id,
        }
        self.actions.push(action);
        id
    }

    pub fn stroke(
        &mut self,
        points: &[Vec2],
        brush: crate::stroke::Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
    ) -> u32 {
        let seed = self.actions.len() as u64 + self.meta.seed;
        let s = Stroke::from_control_points(points, brush, layer, stage, role, seed);
        self.push(Action::Stroke(s))
    }

    pub fn closed_stroke(
        &mut self,
        points: &[Vec2],
        brush: crate::stroke::Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
    ) -> u32 {
        let seed = self.actions.len() as u64 + self.meta.seed;
        let s = Stroke::from_closed_control_points(points, brush, layer, stage, role, seed);
        self.push(Action::Stroke(s))
    }

    pub fn polyline(
        &mut self,
        points: &[Vec2],
        brush: crate::stroke::Brush,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
    ) -> u32 {
        let seed = self.actions.len() as u64 + self.meta.seed;
        let s = Stroke::from_polyline(points, brush, layer, stage, role, seed);
        self.push(Action::Stroke(s))
    }

    pub fn fill(
        &mut self,
        polygon: Vec<Vec2>,
        color: Rgba,
        layer: u8,
        stage: Stage,
        role: impl Into<String>,
    ) -> u32 {
        let id = self.actions.len() as u32 + 1;
        self.push(Action::Fill(FillRegion::new(
            id, layer, stage, polygon, color, role,
        )))
    }

    /// Assign start/duration to every action.
    ///
    /// `speed` is a global multiplier: 1.0 is a natural hand speed, 4.0 is a
    /// timelapse. Timing is derived from path length so long strokes genuinely
    /// take longer, like a real hand.
    pub fn schedule(&mut self, speed: f64) {
        let speed = speed.max(0.05);
        let mut t = 0.0f64;
        let mut last_stage: Option<Stage> = None;
        self.timings.clear();
        for action in &self.actions {
            if last_stage != Some(action.stage()) {
                t += 260.0;
                last_stage = Some(action.stage());
            }
            let dur = match action {
                Action::Fill(_) => 420.0,
                Action::Stroke(s) => {
                    let factor = stage_speed_factor(s.stage);
                    let px_per_s = (s.brush.kind.base_speed() as f64) * factor * speed;
                    ((s.length() as f64 / px_per_s) * 1000.0).clamp(45.0, 5000.0) + 35.0
                }
            };
            self.timings.push(Timing {
                start_ms: t,
                duration_ms: dur,
            });
            t += dur + 32.0;
        }
        self.duration_ms = t;
    }

    pub fn total_ms(&self) -> f64 {
        self.duration_ms
    }

    pub fn stroke_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|a| matches!(a, Action::Stroke(_)))
            .count()
    }

    pub fn fill_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|a| matches!(a, Action::Fill(_)))
            .count()
    }

    pub fn point_count(&self) -> usize {
        self.actions
            .iter()
            .map(|a| match a {
                Action::Stroke(s) => s.points.len(),
                Action::Fill(_) => 0,
            })
            .sum()
    }

    pub fn bounds(&self) -> Bounds {
        let mut b = Bounds::empty();
        for a in &self.actions {
            b.union(a.bounds());
        }
        b
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Stage boundaries as (stage, start_ms) for the timeline UI.
    pub fn stage_marks(&self) -> Vec<(Stage, f64)> {
        let mut out: Vec<(Stage, f64)> = Vec::new();
        for (i, a) in self.actions.iter().enumerate() {
            let st = a.stage();
            if out.last().map(|(s, _)| *s) != Some(st) {
                let t = self.timings.get(i).map(|t| t.start_ms).unwrap_or(0.0);
                out.push((st, t));
            }
        }
        out
    }
}

fn stage_speed_factor(stage: Stage) -> f64 {
    match stage {
        Stage::Composition => 1.35,
        Stage::Rough => 1.2,
        Stage::Construction => 1.25,
        Stage::Face => 0.95,
        Stage::Eyes => 0.8,
        Stage::NoseMouth => 0.9,
        Stage::Hair => 0.9,
        Stage::Body => 1.0,
        Stage::Clothing => 1.0,
        Stage::Lineart => 0.72,
        Stage::Color => 1.5,
        Stage::Shading => 1.05,
        Stage::Correction => 0.85,
    }
}

/// Convenience: does this brush count as lineart ink?
pub fn is_ink(kind: BrushKind) -> bool {
    matches!(kind, BrushKind::Ink | BrushKind::Brush)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stroke::Brush;

    fn doc() -> Document {
        Document::new(
            Canvas::new(400, 400),
            DocMeta::new("t", "p", 1),
        )
    }

    #[test]
    fn scheduling_is_monotonic() {
        let mut d = doc();
        let b = Brush::new(BrushKind::Ink, 2.0, Rgba::rgb(0, 0, 0));
        d.stroke(&[Vec2::new(10.0, 10.0), Vec2::new(300.0, 10.0)], b.clone(), 4, Stage::Lineart, "a");
        d.stroke(&[Vec2::new(10.0, 50.0), Vec2::new(300.0, 50.0)], b, 4, Stage::Lineart, "b");
        d.schedule(1.0);
        assert_eq!(d.timings.len(), 2);
        assert!(d.timings[0].start_ms < d.timings[1].start_ms);
        assert!(d.duration_ms > d.timings[1].start_ms);
    }

    #[test]
    fn progress_is_clamped() {
        let t = Timing { start_ms: 100.0, duration_ms: 200.0 };
        assert_eq!(t.progress(0.0), 0.0);
        assert_eq!(t.progress(1000.0), 1.0);
        assert!((t.progress(200.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ids_are_unique() {
        let mut d = doc();
        let b = Brush::new(BrushKind::Ink, 2.0, Rgba::rgb(0, 0, 0));
        let a = d.stroke(&[Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)], b.clone(), 4, Stage::Lineart, "a");
        let c = d.stroke(&[Vec2::new(2.0, 2.0), Vec2::new(3.0, 3.0)], b, 4, Stage::Lineart, "b");
        assert_ne!(a, c);
    }
}
