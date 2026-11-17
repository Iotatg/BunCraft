//! Stroke prediction models.
//!
//! `StrokeModel` is the seam between the drawing engine and any sequence model.
//! This build ships one concrete implementation: a GRU with the same interface
//! as SketchRNN's decoder (5-D stroke input, 5-D mixture-free output).
//!
//! **No trained weights are bundled.** The inference code is real and tested,
//! but a randomly initialised network draws noise. Training is a separate step
//! (`buncraft-train`, see research/research_gaps.md); the engine never pretends
//! otherwise.

use buncraft_core::rng::Rng;
use serde::{Deserialize, Serialize};

pub const STROKE_DIM: usize = 5;

pub trait StrokeModel {
    fn name(&self) -> &str;
    fn reset(&mut self);
    /// One step: feed a stroke-5 vector, get the next one.
    fn step(&mut self, input: [f32; STROKE_DIM]) -> [f32; STROKE_DIM];
    /// Autoregressive rollout.
    fn generate(&mut self, seed: [f32; STROKE_DIM], steps: usize) -> Vec<[f32; STROKE_DIM]> {
        self.reset();
        let mut out = Vec::with_capacity(steps);
        let mut cur = seed;
        for _ in 0..steps {
            cur = self.step(cur);
            out.push(cur);
        }
        out
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GruWeights {
    pub input_dim: usize,
    pub hidden: usize,
    /// [3*hidden x input_dim]
    pub w_ih: Vec<f32>,
    /// [3*hidden x hidden]
    pub w_hh: Vec<f32>,
    /// [3*hidden]
    pub b: Vec<f32>,
    /// [5 x hidden]
    pub w_out: Vec<f32>,
    /// [5]
    pub b_out: Vec<f32>,
}

impl GruWeights {
    pub fn zeros(hidden: usize) -> Self {
        let h = hidden.max(1);
        Self {
            input_dim: STROKE_DIM,
            hidden: h,
            w_ih: vec![0.0; 3 * h * STROKE_DIM],
            w_hh: vec![0.0; 3 * h * h],
            b: vec![0.0; 3 * h],
            w_out: vec![0.0; STROKE_DIM * h],
            b_out: vec![0.0; STROKE_DIM],
        }
    }

    /// Small random weights. Useful for shape tests and for verifying that the
    /// forward pass is numerically stable; not a usable artist.
    pub fn random(hidden: usize, seed: u64) -> Self {
        let mut w = Self::zeros(hidden);
        let mut rng = Rng::new(seed);
        let scale = (1.0 / hidden as f32).sqrt();
        for v in w.w_ih.iter_mut().chain(w.w_hh.iter_mut()) {
            *v = rng.gauss() * scale * 0.5;
        }
        for v in w.w_out.iter_mut() {
            *v = rng.gauss() * scale;
        }
        w
    }

    pub fn validate(&self) -> Result<(), String> {
        let h = self.hidden;
        if h == 0 || self.input_dim != STROKE_DIM {
            return Err("invalid dimensions".to_string());
        }
        if self.w_ih.len() != 3 * h * STROKE_DIM
            || self.w_hh.len() != 3 * h * h
            || self.b.len() != 3 * h
            || self.w_out.len() != STROKE_DIM * h
            || self.b_out.len() != STROKE_DIM
        {
            return Err("weight tensor size mismatch".to_string());
        }
        Ok(())
    }
}

pub struct SketchRnn {
    w: GruWeights,
    h: Vec<f32>,
    name: String,
}

impl SketchRnn {
    pub fn new(w: GruWeights) -> Result<Self, String> {
        w.validate()?;
        let h = vec![0.0; w.hidden];
        Ok(Self {
            w,
            h,
            name: "sketchrnn-gru".to_string(),
        })
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let w: GruWeights = serde_json::from_str(text).map_err(|e| e.to_string())?;
        Self::new(w)
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&text)
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let text = serde_json::to_string(&self.w).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| e.to_string())
    }

    /// Expose the hidden state for tests / debugging.
    pub fn hidden(&self) -> &[f32] {
        &self.h
    }
}

impl StrokeModel for SketchRnn {
    fn name(&self) -> &str {
        &self.name
    }

    fn reset(&mut self) {
        self.h.iter_mut().for_each(|v| *v = 0.0);
    }

    fn step(&mut self, input: [f32; STROKE_DIM]) -> [f32; STROKE_DIM] {
        let h = self.w.hidden;
        let mut z = vec![0.0; h];
        let mut r = vec![0.0; h];
        let mut n = vec![0.0; h];

        // gates: rows [0,h) = z, [h,2h) = r, [2h,3h) = candidate
        for i in 0..h {
            let mut zi = self.w.b[i];
            let mut ri = self.w.b[h + i];
            for k in 0..STROKE_DIM {
                zi += self.w.w_ih[i * STROKE_DIM + k] * input[k];
                ri += self.w.w_ih[(h + i) * STROKE_DIM + k] * input[k];
            }
            for j in 0..h {
                zi += self.w.w_hh[i * h + j] * self.h[j];
                ri += self.w.w_hh[(h + i) * h + j] * self.h[j];
            }
            z[i] = sigmoid(zi);
            r[i] = sigmoid(ri);
        }

        for i in 0..h {
            let mut ni = self.w.b[2 * h + i];
            for k in 0..STROKE_DIM {
                ni += self.w.w_ih[(2 * h + i) * STROKE_DIM + k] * input[k];
            }
            for j in 0..h {
                ni += self.w.w_hh[(2 * h + i) * h + j] * (r[j] * self.h[j]);
            }
            n[i] = ni.tanh();
        }

        for i in 0..h {
            self.h[i] = (1.0 - z[i]) * n[i] + z[i] * self.h[i];
        }

        let mut out = [0.0f32; STROKE_DIM];
        for k in 0..STROKE_DIM {
            let mut o = self.w.b_out[k];
            for i in 0..h {
                o += self.w.w_out[k * h + i] * self.h[i];
            }
            out[k] = o;
        }
        out
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_weights_keep_hidden_at_zero() {
        let mut m = SketchRnn::new(GruWeights::zeros(8)).unwrap();
        let out = m.step([0.5, -0.2, 1.0, 0.0, 0.0]);
        assert!(out.iter().all(|v| v.abs() < 1e-6));
        assert!(m.hidden().iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn random_weights_stay_finite() {
        let mut m = SketchRnn::new(GruWeights::random(32, 99)).unwrap();
        let seq = m.generate([0.1, 0.2, 1.0, 0.0, 0.0], 64);
        assert_eq!(seq.len(), 64);
        for s in &seq {
            assert!(s.iter().all(|v| v.is_finite()));
            assert!(s.iter().all(|v| v.abs() < 1e6));
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut m = SketchRnn::new(GruWeights::random(16, 3)).unwrap();
        m.step([0.3, 0.1, 1.0, 0.0, 0.0]);
        assert!(m.hidden().iter().any(|v| v.abs() > 1e-9));
        m.reset();
        assert!(m.hidden().iter().all(|v| *v == 0.0));
    }

    #[test]
    fn checkpoint_roundtrip() {
        let w = GruWeights::random(8, 5);
        let m = SketchRnn::new(w.clone()).unwrap();
        let text = serde_json::to_string(&w).unwrap();
        let m2 = SketchRnn::from_json(&text).unwrap();
        assert_eq!(m.hidden().len(), m2.hidden().len());
    }

    #[test]
    fn rejects_broken_checkpoint() {
        let mut w = GruWeights::zeros(4);
        w.w_out.pop();
        assert!(SketchRnn::new(w).is_err());
    }
}
