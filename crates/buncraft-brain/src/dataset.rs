//! Stroke dataset I/O.
//!
//! BunCraft uses the stroke-5 convention (the same one QuickDraw/SketchRNN use):
//! each point is `[dx, dy, pen_down, stroke_end, sketch_end]`.
//!
//! Input files are JSONL, one sketch per line:
//! `{"label":"face","strokes":[[[10,20],[12,24]],[[30,40],[32,42]]]}`
//!
//! Coordinates are absolute pixels in the source; `to_stroke5` converts them to
//! relative offsets and appends the pen state flags.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawSample {
    pub strokes: Vec<Vec<[f32; 2]>>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Stroke5Sample {
    pub points: Vec<[f32; 5]>,
    pub label: Option<String>,
    pub stroke_count: usize,
    pub point_count: usize,
}

pub fn parse_jsonl(text: &str) -> Result<Vec<RawSample>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let sample: RawSample =
            serde_json::from_str(line).map_err(|e| format!("line {}: {}", i + 1, e))?;
        out.push(sample);
    }
    Ok(out)
}

pub fn load_jsonl(path: impl AsRef<std::path::Path>) -> Result<Vec<RawSample>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_jsonl(&text)
}

/// Convert absolute polylines into stroke-5 training vectors.
pub fn to_stroke5(sample: &RawSample) -> Stroke5Sample {
    let mut points = Vec::new();
    let mut point_count = 0;
    let n_strokes = sample.strokes.len();
    for (si, stroke) in sample.strokes.iter().enumerate() {
        let mut prev = [0.0f32, 0.0];
        for (pi, p) in stroke.iter().enumerate() {
            let dx = p[0] - prev[0];
            let dy = p[1] - prev[1];
            prev = [p[0], p[1]];
            let stroke_end = if pi + 1 == stroke.len() { 1.0 } else { 0.0 };
            let sketch_end = if si + 1 == n_strokes && pi + 1 == stroke.len() {
                1.0
            } else {
                0.0
            };
            points.push([dx, dy, 1.0, stroke_end, sketch_end]);
            point_count += 1;
        }
    }
    Stroke5Sample {
        points,
        label: sample.label.clone(),
        stroke_count: n_strokes,
        point_count,
    }
}

/// Scale absolute coordinates into `0..1` on both axes (aspect preserved).
pub fn normalise(sample: &mut RawSample, width: f32, height: f32) {
    let scale = width.max(height).max(1.0);
    for stroke in &mut sample.strokes {
        for p in stroke.iter_mut() {
            p[0] /= scale;
            p[1] /= scale;
        }
    }
}

/// Split into train/validation by index, deterministically.
pub fn split(samples: Vec<RawSample>, val_ratio: f32) -> (Vec<RawSample>, Vec<RawSample>) {
    if samples.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let n_val = ((samples.len() as f32) * val_ratio.clamp(0.0, 0.9)).round() as usize;
    let split_at = samples.len() - n_val.min(samples.len());
    let mut train = samples;
    let val = train.split_off(split_at);
    (train, val)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RawSample {
        RawSample {
            strokes: vec![
                vec![[10.0, 10.0], [20.0, 10.0]],
                vec![[30.0, 30.0], [30.0, 40.0]],
            ],
            label: Some("face".to_string()),
        }
    }

    #[test]
    fn stroke5_offsets_are_relative() {
        let s = to_stroke5(&sample());
        assert_eq!(s.points.len(), 4);
        assert_eq!(s.points[0][0], 10.0);
        assert_eq!(s.points[1][0], 10.0);
        // second stroke starts relative to origin, not to the previous stroke
        assert_eq!(s.points[2][0], 30.0);
        assert_eq!(s.points[2][1], 30.0);
    }

    #[test]
    fn pen_flags_mark_stroke_and_sketch_end() {
        let s = to_stroke5(&sample());
        assert_eq!(s.points[1][3], 1.0); // end of first stroke
        assert_eq!(s.points[1][4], 0.0);
        assert_eq!(s.points[3][3], 1.0);
        assert_eq!(s.points[3][4], 1.0); // end of sketch
    }

    #[test]
    fn parses_jsonl_with_blank_lines() {
        let text = "\n{\"strokes\":[[[0,0],[1,1]]]}\n\n";
        let v = parse_jsonl(text).unwrap();
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn split_is_deterministic() {
        let v: Vec<RawSample> = (0..10).map(|_| sample()).collect();
        let (a, b) = split(v.clone(), 0.2);
        assert_eq!(a.len(), 8);
        assert_eq!(b.len(), 2);
    }
}
