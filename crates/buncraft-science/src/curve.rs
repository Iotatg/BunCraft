//! True curve geometry: analytic derivatives, adaptive Gauss–Legendre arc
//! length, and inverse arc length.
//!
//! BunCraft's `catmull_rom` samples uniformly in `t` (10 points per segment)
//! and `resample` spaces points by chord length of that polyline. Neither knows
//! the analytic speed |C'(t)|, so a fast gesture and a slow one get the same
//! samples. Arc-length parametrisation is what makes pen speed physical.
//!
//! Independent implementation. The upstream reference for the *approach* is
//! `upstream/kurbo/kurbo/src/param_curve.rs` and `cubicbez.rs:628-675`; no code
//! is copied.

pub type Pt = [f64; 2];

fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn add(a: Pt, b: Pt) -> Pt {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: Pt, b: Pt) -> Pt {
    [a[0] - b[0], a[1] - b[1]]
}

fn scale(a: Pt, s: f64) -> Pt {
    [a[0] * s, a[1] * s]
}

fn hypot(v: Pt) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// 8-point Gauss–Legendre nodes/weights on [-1, 1] (even symmetry stored once).
/// Standard quadrature table; used here to integrate |C'(t)|.
const GL8: [(f64, f64); 4] = [
    (0.1834346424956498, 0.3626837833783620),
    (0.5255324099163290, 0.3137066458778873),
    (0.7966664774136267, 0.2223810344533745),
    (0.9602898564975363, 0.1012285362903763),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBez {
    pub p0: Pt,
    pub p1: Pt,
    pub p2: Pt,
    pub p3: Pt,
}

/// A cubic with `p1 = p0 + (p3-p0)/3`, `p2 = p0 + 2(p3-p0)/3` is exactly a
/// straight line with constant speed |p3-p0|.
impl CubicBez {
    pub fn new(p0: Pt, p1: Pt, p2: Pt, p3: Pt) -> Self {
        Self { p0, p1, p2, p3 }
    }

    pub fn line(a: Pt, b: Pt) -> Self {
        Self::new(a, lerp(a, b, 1.0 / 3.0), lerp(a, b, 2.0 / 3.0), b)
    }

    /// Bernstein / de Casteljau evaluation at t ∈ [0,1].
    pub fn eval(&self, t: f64) -> Pt {
        let mt = 1.0 - t;
        let a = add(scale(self.p0, mt * mt * mt), scale(self.p1, 3.0 * mt * mt * t));
        let b = add(scale(self.p2, 3.0 * mt * t * t), scale(self.p3, t * t * t));
        add(a, b)
    }

    /// Analytic first derivative: 3[(1-t)²(p1-p0) + 2(1-t)t(p2-p1) + t²(p3-p2)].
    pub fn deriv(&self, t: f64) -> Pt {
        let mt = 1.0 - t;
        let a = scale(sub(self.p1, self.p0), 3.0 * mt * mt);
        let b = scale(sub(self.p2, self.p1), 6.0 * mt * t);
        let c = scale(sub(self.p3, self.p2), 3.0 * t * t);
        add(add(a, b), c)
    }

    /// Curvature κ = |x'y'' − y'x''| / |C'|³, from the analytic second derivative.
    pub fn curvature(&self, t: f64) -> f64 {
        let d = self.deriv(t);
        let dd = [
            6.0 * ((1.0 - t) * (self.p2[0] - 2.0 * self.p1[0] + self.p0[0])
                + t * (self.p3[0] - 2.0 * self.p2[0] + self.p1[0])),
            6.0 * ((1.0 - t) * (self.p2[1] - 2.0 * self.p1[1] + self.p0[1])
                + t * (self.p3[1] - 2.0 * self.p2[1] + self.p1[1])),
        ];
        let speed = hypot(d);
        if speed < 1e-12 {
            return 0.0;
        }
        (d[0] * dd[1] - d[1] * dd[0]).abs() / (speed * speed * speed)
    }

    /// de Casteljau split at t. Left and right halves share the split point.
    pub fn split(&self, t: f64) -> (Self, Self) {
        let a = lerp(self.p0, self.p1, t);
        let b = lerp(self.p1, self.p2, t);
        let c = lerp(self.p2, self.p3, t);
        let d = lerp(a, b, t);
        let e = lerp(b, c, t);
        let f = lerp(d, e, t);
        (
            Self::new(self.p0, a, d, f),
            Self::new(f, e, c, self.p3),
        )
    }

    /// Sub-curve over [t0, t1].
    pub fn subsegment(&self, t0: f64, t1: f64) -> Self {
        let t0 = t0.clamp(0.0, 1.0);
        let t1 = t1.clamp(0.0, 1.0);
        if t1 <= t0 {
            return Self::new(self.eval(t0), self.eval(t0), self.eval(t0), self.eval(t0));
        }
        let (_, right) = self.split(t0);
        let u = ((t1 - t0) / (1.0 - t0)).clamp(0.0, 1.0);
        right.split(u).0
    }

    /// Gauss–Legendre estimate of ∫₀¹ |C'(t)| dt.
    fn gauss8(&self) -> f64 {
        let mut sum = 0.0;
        for &(x, w) in &GL8 {
            sum += w * (hypot(self.deriv(0.5 * (x + 1.0))) + hypot(self.deriv(0.5 * (1.0 - x))));
        }
        // Mapping t = (x+1)/2 gives dt = dx/2; the stored weight pairs already
        // sum to 2 over [-1,1], so the correct prefactor is 1/2, not 1/4.
        0.5 * sum
    }

    /// Adaptive arc length: compare the whole-segment Gauss estimate with the
    /// sum of two half-segment estimates, recurse until the difference is
    /// within `accuracy` (or depth 20, matching kurbo's safety limit).
    pub fn arclen(&self, accuracy: f64) -> f64 {
        self.arclen_refine(accuracy.max(1e-12), 0)
    }

    fn arclen_refine(&self, accuracy: f64, depth: usize) -> f64 {
        let whole = self.gauss8();
        let (l, r) = self.split(0.5);
        let halves = l.gauss8() + r.gauss8();
        if (whole - halves).abs() < accuracy || depth >= 20 {
            return halves;
        }
        l.arclen_refine(accuracy * 0.5, depth + 1) + r.arclen_refine(accuracy * 0.5, depth + 1)
    }

    /// Inverse arc length: the parameter t whose arc length from 0 is `s`.
    /// Newton on the monotone function S(t) = arc(0..t), with bisection
    /// fallback so it also works where |C'| ≈ 0.
    pub fn inv_arclen(&self, s: f64, accuracy: f64) -> f64 {
        let total = self.arclen(accuracy);
        self.inv_arclen_with_total(s, total, accuracy)
    }

    /// Same as [`CubicBez::inv_arclen`], but reuses a precomputed total length.
    /// Sampling `n` points from one segment would otherwise recompute the
    /// segment's arc length `n` times.
    pub fn inv_arclen_with_total(&self, s: f64, total: f64, accuracy: f64) -> f64 {
        if s <= 0.0 {
            return 0.0;
        }
        if s >= total {
            return 1.0;
        }
        let mut t = (s / total).clamp(0.0, 1.0);
        let mut lo = 0.0f64;
        let mut hi = 1.0f64;
        for _ in 0..32 {
            let arc = self.subsegment(0.0, t).arclen(accuracy);
            let err = arc - s;
            if err.abs() < accuracy.max(1e-9) {
                return t;
            }
            if err > 0.0 {
                hi = t;
            } else {
                lo = t;
            }
            let speed = hypot(self.deriv(t));
            let mut next = if speed > 1e-9 { t - err / speed } else { 0.5 * (lo + hi) };
            if !(lo..=hi).contains(&next) || !next.is_finite() {
                next = 0.5 * (lo + hi);
            }
            t = next;
        }
        t
    }

    /// Evenly spaced samples **by arc length** — what a pressure/speed-aware
    /// stroke model needs, as opposed to uniform t.
    pub fn sample_by_arclen(&self, n: usize, accuracy: f64) -> Vec<Pt> {
        let total = self.arclen(accuracy);
        (0..=n.max(1))
            .map(|i| {
                let s = total * i as f64 / n.max(1) as f64;
                self.eval(self.inv_arclen(s, accuracy))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ∫₀¹ sqrt(1+4t²) dt = sqrt(5)/2 + ln(2+sqrt(5))/4  (the curve x=t, y=t²).
    fn parabola() -> CubicBez {
        CubicBez::new([0.0, 0.0], [1.0 / 3.0, 0.0], [2.0 / 3.0, 1.0 / 3.0], [1.0, 1.0])
    }

    #[test]
    fn line_arclen_is_exact() {
        let c = CubicBez::line([3.0, -2.0], [13.0, -2.0]);
        assert!((c.arclen(1e-10) - 10.0).abs() < 1e-9);
        assert!(c.curvature(0.5).abs() < 1e-9);
        assert!((hypot(c.deriv(0.5)) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn arclen_matches_closed_form() {
        let expected = 5.0f64.sqrt() * 0.5 + (2.0 + 5.0f64.sqrt()).ln() * 0.25;
        let got = parabola().arclen(1e-11);
        assert!((got - expected).abs() < 1e-7, "got {got}, expected {expected}");
    }

    #[test]
    fn split_is_continuous() {
        let c = parabola();
        let (l, r) = c.split(0.37);
        assert!((l.eval(1.0)[0] - r.eval(0.0)[0]).abs() < 1e-12);
        assert!((l.eval(1.0)[1] - r.eval(0.0)[1]).abs() < 1e-12);
        assert!((l.eval(1.0)[0] - c.eval(0.37)[0]).abs() < 1e-12);
    }

    #[test]
    fn inverse_arclen_inverts_arclen() {
        let c = parabola();
        let total = c.arclen(1e-11);
        for k in 1..10 {
            let s = total * k as f64 / 10.0;
            let t = c.inv_arclen(s, 1e-9);
            let back = c.subsegment(0.0, t).arclen(1e-11);
            assert!((back - s).abs() < 2e-6, "k={k} t={t} back={back} s={s}");
        }
    }

    #[test]
    fn uniform_t_is_not_arclen() {
        let c = parabola();
        let half = c.arclen(1e-11) * 0.5;
        let t_half = c.inv_arclen(half, 1e-9);
        // speed grows with t, so half the length is reached after t = 0.5
        assert!(t_half > 0.55, "t_half={t_half}");
        // arc-length samples are evenly spaced in distance, unlike uniform t
        let pts = c.sample_by_arclen(10, 1e-9);
        let dists: Vec<f64> = pts.windows(2).map(|w| hypot(sub(w[1], w[0]))).collect();
        let mean = dists.iter().sum::<f64>() / dists.len() as f64;
        let uniform: Vec<f64> = (0..=10)
            .map(|i| c.eval(i as f64 / 10.0))
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| hypot(sub(w[1], w[0])))
            .collect();
        let spread_arc = dists.iter().map(|d| (d - mean).abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let spread_t = uniform.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            - uniform.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!(spread_arc < spread_t, "arc spread {spread_arc}, t spread {spread_t}");
    }
}
