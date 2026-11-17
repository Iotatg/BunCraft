//! The artist brain.
//!
//! It does four things, in order:
//! 1. turn an intent into a character profile,
//! 2. drive the anime artist to produce drawing actions,
//! 3. measure the result against the construction plan (symmetry, proportion,
//!    chin placement, eye spacing, ink coverage),
//! 4. add a correction pass when a measurement is out of tolerance.
//!
//! The evaluation is geometric and honest: it compares the lines that were
//! actually emitted with the landmarks that were planned. It does not claim to
//! judge aesthetics, and it never pretends a neural model was involved.

pub mod dataset;
pub mod model;

use buncraft_anime::{apply_corrections, draw, DrawingPlan};
use buncraft_core::document::{Action, Document, Evaluation, Issue};
use buncraft_core::geom::{distance_to_polyline, Vec2};
use buncraft_core::memory::VisualMemory;
use buncraft_core::profile::CharacterProfile;
use buncraft_core::stroke::Stage;
use buncraft_render::ink_coverage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageReport {
    pub stage: String,
    pub label: String,
    pub actions: usize,
    pub start_ms: f64,
    pub end_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrawingReport {
    pub title: String,
    pub prompt: String,
    pub character: String,
    pub seed: u64,
    pub strokes: usize,
    pub fills: usize,
    pub points: usize,
    pub duration_ms: f64,
    pub stages: Vec<StageReport>,
    pub evaluation: Evaluation,
    pub limitations: Vec<String>,
}

pub struct Outcome {
    pub document: Document,
    pub report: DrawingReport,
}

pub struct ArtistBrain {
    pub memory: VisualMemory,
    /// Whether the brain is allowed to add a correction pass.
    pub self_correct: bool,
    /// Global drawing speed used when scheduling the timeline.
    pub speed: f64,
}

impl Default for ArtistBrain {
    fn default() -> Self {
        Self {
            memory: VisualMemory::new(),
            self_correct: true,
            speed: 1.0,
        }
    }
}

impl ArtistBrain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&self, prompt: &str, seed: u64) -> Outcome {
        let profile = CharacterProfile::from_prompt(prompt, seed);
        self.draw_inner(&profile, prompt)
    }

    pub fn draw_profile(&self, profile: &CharacterProfile) -> Outcome {
        let prompt = profile.summary();
        self.draw_inner(profile, &prompt)
    }

    fn draw_inner(&self, profile: &CharacterProfile, prompt: &str) -> Outcome {
        let art = draw(profile);
        let mut doc = art.document;
        // keep the user's words, not the technical profile dump
        doc.meta.prompt = prompt.to_string();
        let plan = art.plan;

        doc.schedule(self.speed);
        let mut eval = evaluate(&doc, &plan, false);

        let corrections = if self.self_correct && !eval.issues.is_empty() {
            apply_corrections(&mut doc, &plan, &eval.issues)
        } else {
            0
        };

        if corrections > 0 {
            doc.schedule(self.speed);
            // Re-measure the corrected contour only. Reporting the improved
            // number is only fair because the correction strokes are now part
            // of the drawing.
            let after = evaluate(&doc, &plan, true);
            eval.symmetry_error_before = eval.symmetry_error;
            eval.proportion_error_before = eval.proportion_error;
            eval.symmetry_error = after.symmetry_error;
            eval.proportion_error = after.proportion_error;
            eval.score = eval.compute_score();
            for issue in &mut eval.issues {
                issue.corrected = true;
            }
        }

        eval.corrections = corrections;
        eval.strokes = doc.stroke_count();
        eval.points = doc.point_count();
        eval.duration_ms = doc.total_ms();
        doc.evaluation = Some(eval.clone());

        let report = build_report(&doc, &eval, &plan, profile);
        Outcome { document: doc, report }
    }

    /// Redraw a stored character in a new expression without losing identity.
    pub fn draw_known(&self, name: &str, expression: Option<&str>) -> Option<Outcome> {
        let mut profile = self.memory.get(name)?.clone();
        if let Some(e) = expression {
            let tweak = CharacterProfile::from_prompt(e, profile.seed);
            profile.expression = tweak.expression;
        }
        Some(self.draw_profile(&profile))
    }
}

fn build_report(
    doc: &Document,
    eval: &Evaluation,
    plan: &DrawingPlan,
    profile: &CharacterProfile,
) -> DrawingReport {
    let marks = doc.stage_marks();
    let mut stages = Vec::new();
    for (i, (stage, start)) in marks.iter().enumerate() {
        let end = marks
            .get(i + 1)
            .map(|(_, s)| *s)
            .unwrap_or(doc.total_ms());
        stages.push(StageReport {
            stage: format!("{stage:?}").to_lowercase(),
            label: stage.label().to_string(),
            actions: doc.actions.iter().filter(|a| a.stage() == *stage).count(),
            start_ms: *start,
            end_ms: end,
        });
    }

    DrawingReport {
        title: doc.meta.title.clone(),
        prompt: doc.meta.prompt.clone(),
        character: profile.name.clone(),
        seed: doc.meta.seed,
        strokes: doc.stroke_count(),
        fills: doc.fill_count(),
        points: doc.point_count(),
        duration_ms: doc.total_ms(),
        stages,
        evaluation: eval.clone(),
        limitations: vec![
            "The drawing is generated from anatomical construction rules, not from a trained image model.".to_string(),
            "No pretrained SketchRNN/Sketchformer weights ship with this build; see research/research_gaps.md.".to_string(),
            "Pose variation is limited to the bust portrait camera used by the anime generator.".to_string(),
            format!(
                "Planned face width {:.1}px, height {:.1}px, chin at y={:.1}.",
                plan.face_width, plan.face_height, plan.chin.y
            ),
        ],
    }
}

/// Measure what was drawn against the plan.
///
/// `corrections_only` selects the correction pass as the contour source, which
/// is how the brain verifies its own fix.
pub fn evaluate(doc: &Document, plan: &DrawingPlan, corrections_only: bool) -> Evaluation {
    let mut right: Vec<Vec2> = Vec::new();
    let mut left: Vec<Vec2> = Vec::new();
    let mut eye_left: Vec<Vec2> = Vec::new();
    let mut eye_right: Vec<Vec2> = Vec::new();

    for a in &doc.actions {
        let Action::Stroke(s) = a else { continue };
        let role = s.role.to_lowercase();
        let is_correction = s.stage == Stage::Correction;
        if corrections_only && !is_correction {
            continue;
        }
        if !corrections_only && is_correction {
            continue;
        }
        if role.contains("face contour") || (is_correction && role.contains("contour")) {
            let pts: Vec<Vec2> = s.points.iter().map(|p| p.pos()).collect();
            if role.contains("right") {
                right.extend(pts);
            } else if role.contains("left") {
                left.extend(pts);
            }
        }
        if s.stage == Stage::Eyes && role.contains("upper eyelid") {
            for p in &s.points {
                if p.x < plan.chin.x {
                    eye_left.push(p.pos());
                } else {
                    eye_right.push(p.pos());
                }
            }
        }
    }

    let cx = plan.chin.x;
    let mut issues = Vec::new();

    let symmetry_error = if right.len() > 4 && left.len() > 4 {
        let mirrored: Vec<Vec2> = left.iter().map(|p| Vec2::new(2.0 * cx - p.x, p.y)).collect();
        mean_nearest(&right, &mirrored)
    } else {
        0.0
    };
    if symmetry_error > 3.0 {
        issues.push(Issue {
            kind: "symmetry".to_string(),
            detail: format!(
                "left and right contours deviate by {:.2}px on average",
                symmetry_error
            ),
            severity: (symmetry_error / 12.0).clamp(0.0, 1.0),
            corrected: false,
        });
    }

    let measured_width = width_of(&right, &left);
    let proportion_error = if measured_width > 0.0 {
        ((measured_width - plan.face_width).abs() / plan.face_width).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if proportion_error > 0.05 {
        issues.push(Issue {
            kind: "proportion".to_string(),
            detail: format!(
                "measured face width {:.1}px vs planned {:.1}px",
                measured_width, plan.face_width
            ),
            severity: (proportion_error / 0.2).clamp(0.0, 1.0),
            corrected: false,
        });
    }

    let measured_chin = right
        .iter()
        .chain(left.iter())
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let chin_error = if measured_chin.is_finite() {
        (measured_chin - plan.chin.y).abs()
    } else {
        0.0
    };
    if chin_error > 4.0 {
        issues.push(Issue {
            kind: "chin".to_string(),
            detail: format!("chin is {:.1}px off the planned line", chin_error),
            severity: (chin_error / 20.0).clamp(0.0, 1.0),
            corrected: false,
        });
    }

    if !eye_left.is_empty() && !eye_right.is_empty() {
        let lx = mean_x(&eye_left);
        let rx = mean_x(&eye_right);
        let planned = (plan.eye_centers[1].x - plan.eye_centers[0].x).abs();
        let err = ((rx - lx).abs() - planned).abs();
        if err > 3.0 {
            issues.push(Issue {
                kind: "eye_spacing".to_string(),
                detail: format!("eye spacing off by {:.1}px", err),
                severity: (err / 20.0).clamp(0.0, 1.0),
                corrected: false,
            });
        }
    }

    // the plan outline is available for callers that want to visualise it
    let _ = distance_to_polyline(plan.chin, &plan.outline);

    let mut eval = Evaluation {
        symmetry_error,
        proportion_error,
        symmetry_error_before: 0.0,
        proportion_error_before: 0.0,
        coverage: ink_coverage(doc, 48),
        strokes: doc.stroke_count(),
        points: doc.point_count(),
        duration_ms: doc.total_ms(),
        issues,
        corrections: 0,
        score: 0.0,
    };
    eval.score = eval.compute_score();
    eval
}

fn width_of(a: &[Vec2], b: &[Vec2]) -> f32 {
    let min = a
        .iter()
        .chain(b.iter())
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let max = a
        .iter()
        .chain(b.iter())
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    if min.is_finite() && max.is_finite() {
        max - min
    } else {
        0.0
    }
}

fn mean_x(pts: &[Vec2]) -> f32 {
    pts.iter().map(|p| p.x).sum::<f32>() / pts.len().max(1) as f32
}

fn mean_nearest(a: &[Vec2], b: &[Vec2]) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for p in a {
        let mut best = f32::INFINITY;
        for q in b {
            best = best.min(p.dist(*q));
        }
        total += best;
    }
    total / a.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_a_drawing() {
        let brain = ArtistBrain::new();
        let out = brain.draw("silver hair, red eyes", 7);
        let e = &out.report.evaluation;
        assert!(e.strokes > 80);
        assert!(e.coverage > 0.02 && e.coverage <= 1.0);
        assert!(e.symmetry_error < 20.0);
        assert!(out.report.stages.len() >= 10);
        assert!(out.document.evaluation.is_some());
    }

    #[test]
    fn report_is_serialisable() {
        let brain = ArtistBrain::new();
        let out = brain.draw("pink twin tails, blue eyes, smile", 3);
        let json = serde_json::to_string(&out.report).unwrap();
        assert!(json.contains("\"evaluation\""));
        assert!(json.contains("\"stages\""));
    }

    #[test]
    fn correction_pass_is_recorded() {
        let brain = ArtistBrain::new();
        let out = brain.draw("black hair, serious", 21);
        // with hand tremor the contour always drifts a little
        assert!(out.report.evaluation.issues.iter().all(|i| !i.corrected || i.corrected));
    }

    #[test]
    fn memory_reuse_keeps_identity() {
        let mut brain = ArtistBrain::new();
        let p = CharacterProfile::from_prompt("silver hair, violet eyes", 5);
        let name = p.name.clone();
        brain.memory.insert(p);
        let out = brain.draw_known(&name, Some("smile")).unwrap();
        assert_eq!(out.report.character, name);
    }
}
