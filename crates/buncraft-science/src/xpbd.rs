//! Position-based dynamics with XPBD compliance, for hair-strand simulation.
//!
//! Equation (18) of Macklin, Müller & Chentanez, *XPBD: Position-Based
//! Simulation of Compliant Constrained Dynamics*, MiG '16:
//!
//! > Δλ = (−C − α̃λ) / (∇C M⁻¹ ∇Cᵀ + α̃),  α̃ = α/Δt²
//!
//! Verified against `upstream/PositionBasedDynamics/PositionBasedDynamics/XPBD.cpp:36-56`
//! (there `stiffness k` is used, so α̃ = 1/(kΔt²)) and
//! `upstream/avian/src/dynamics/solver/xpbd/mod.rs:424-426`
//! (`tilde_compliance = compliance / dt.powi(2)`).
//!
//! Independent implementation. In particular λ is reset to zero at the start
//! of **every** time step — XPBD multipliers are per-step, not persistent.
//! α̃ = 0 (compliance = 0) recovers ordinary PBD, whose stiffness depends on
//! iteration count; XPBD removes that dependency.

pub type P3 = [f64; 3];

fn add(a: P3, b: P3) -> P3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: P3, s: f64) -> P3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: P3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: P3,
    pub prev: P3,
    pub vel: P3,
    /// 0 means pinned (infinite mass).
    pub inv_mass: f64,
}

impl Particle {
    pub fn new(pos: P3, inv_mass: f64) -> Self {
        Self { pos, prev: pos, vel: [0.0; 3], inv_mass }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DistanceConstraint {
    pub a: usize,
    pub b: usize,
    pub rest: f64,
    /// α = 1/k, units m/N. 0 = infinitely stiff (plain PBD).
    pub compliance: f64,
    /// Per-step Lagrange multiplier, reset each time step.
    pub lambda: f64,
}

impl DistanceConstraint {
    pub fn new(a: usize, b: usize, rest: f64, compliance: f64) -> Self {
        Self { a, b, rest, compliance, lambda: 0.0 }
    }
}

/// Solve one XPBD distance constraint in place. Returns (C, Δλ) for diagnosis.
/// n points from b to a, matching `XPBD.cpp:22-23` (`n = p0 - p1`).
pub fn solve_distance_xpbd(
    c: &mut DistanceConstraint,
    particles: &mut [Particle],
    dt: f64,
) -> (f64, f64) {
    if c.a == c.b || c.a >= particles.len() || c.b >= particles.len() {
        return (0.0, 0.0);
    }
    let (pa, pb) = (particles[c.a].pos, particles[c.b].pos);
    let w0 = particles[c.a].inv_mass;
    let w1 = particles[c.b].inv_mass;
    let mut n = sub(pa, pb);
    let d = len(n);
    if d <= 1e-9 || (w0 + w1) <= 0.0 {
        return (0.0, 0.0);
    }
    n = scale(n, 1.0 / d);
    let big_c = d - c.rest;
    let alpha_tilde = c.compliance / (dt * dt);
    let k = w0 + w1 + alpha_tilde;
    let d_lambda = (-big_c - alpha_tilde * c.lambda) / k;
    c.lambda += d_lambda;
    let impulse = scale(n, d_lambda);
    particles[c.a].pos = add(pa, scale(impulse, w0));
    particles[c.b].pos = sub(pb, scale(impulse, w1));
    (big_c, d_lambda)
}

/// A chain of particles with stretch links and simplified bending links.
/// Bending is modelled as a distance constraint between i−1 and i+1 with
/// rest = 2·segment length (a straight configuration). The authoritative
/// upstream alternative is the dihedral/elastic-rod constraint in
/// `PositionBasedDynamics/Simulation/Constraints.cpp:1265+` and
/// `PositionBasedElasticRods.cpp:1188-1202`; that is angle-based and more
/// accurate, this is the cheap triangle approximation.
#[derive(Clone, Debug)]
pub struct HairStrand {
    pub particles: Vec<Particle>,
    pub links: Vec<DistanceConstraint>,
    pub bends: Vec<DistanceConstraint>,
    pub gravity: P3,
    pub damping: f64,
}

impl HairStrand {
    /// Build a vertical strand of `n` particles, `seg` apart, root pinned.
    pub fn new(root: P3, n: usize, seg: f64, compliance: f64, bend_compliance: f64) -> Self {
        let mut particles = Vec::with_capacity(n);
        for i in 0..n {
            let p = [root[0], root[1] - i as f64 * seg, root[2]];
            particles.push(Particle::new(p, if i == 0 { 0.0 } else { 1.0 }));
        }
        let mut links = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            links.push(DistanceConstraint::new(i, i + 1, seg, compliance));
        }
        let mut bends = Vec::with_capacity(n.saturating_sub(2));
        for i in 1..n - 1 {
            bends.push(DistanceConstraint::new(i - 1, i + 1, 2.0 * seg, bend_compliance));
        }
        Self { particles, links, bends, gravity: [0.0, -9.81, 0.0], damping: 0.8 }
    }

    /// One physics step: substepped semi-implicit Euler + Gauss–Seidel
    /// constraint projection, with XPBD velocity update v = (x − x_prev)/h.
    pub fn step(&mut self, dt: f64, substeps: usize, iterations: usize) {
        let substeps = substeps.max(1);
        let h = dt / substeps as f64;
        for _ in 0..substeps {
            for p in self.particles.iter_mut() {
                if p.inv_mass == 0.0 {
                    p.prev = p.pos;
                    continue;
                }
                p.prev = p.pos;
                p.vel = add(p.vel, scale(self.gravity, h));
                p.vel = scale(p.vel, (1.0 - self.damping * h).max(0.0));
                p.pos = add(p.pos, scale(p.vel, h));
            }
            // XPBD multipliers live for exactly one time step.
            for c in self.links.iter_mut().chain(self.bends.iter_mut()) {
                c.lambda = 0.0;
            }
            for _ in 0..iterations {
                for i in 0..self.links.len() {
                    let mut c = self.links[i];
                    solve_distance_xpbd(&mut c, &mut self.particles, h);
                    self.links[i] = c;
                }
                for i in 0..self.bends.len() {
                    let mut c = self.bends[i];
                    solve_distance_xpbd(&mut c, &mut self.particles, h);
                    self.bends[i] = c;
                }
            }
            for p in self.particles.iter_mut() {
                if p.inv_mass > 0.0 {
                    p.vel = scale(sub(p.pos, p.prev), 1.0 / h);
                }
            }
        }
    }

    pub fn tip(&self) -> P3 {
        self.particles.last().map(|p| p.pos).unwrap_or([0.0; 3])
    }

    pub fn max_stretch(&self) -> f64 {
        self.links
            .iter()
            .map(|c| (len(sub(self.particles[c.a].pos, self.particles[c.b].pos)) - c.rest).abs())
            .fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stiff_distance_constraint_reaches_rest_in_one_solve() {
        let mut ps = [Particle::new([0.0, 0.0, 0.0], 1.0), Particle::new([2.0, 0.0, 0.0], 1.0)];
        let mut c = DistanceConstraint::new(0, 1, 1.0, 0.0);
        let (big_c, dl) = solve_distance_xpbd(&mut c, &mut ps, 1.0 / 60.0);
        assert!((big_c - 1.0).abs() < 1e-12);
        assert!(dl < 0.0);
        let d = len(sub(ps[0].pos, ps[1].pos));
        assert!((d - 1.0).abs() < 1e-12, "d={d}");
    }

    #[test]
    fn compliance_limits_the_correction() {
        let solve = |compliance: f64| {
            let mut ps = [Particle::new([0.0, 0.0, 0.0], 1.0), Particle::new([2.0, 0.0, 0.0], 1.0)];
            let mut c = DistanceConstraint::new(0, 1, 1.0, compliance);
            let (_, dl) = solve_distance_xpbd(&mut c, &mut ps, 1.0 / 60.0);
            (len(sub(ps[0].pos, ps[1].pos)), dl)
        };
        let (d_stiff, _) = solve(0.0);
        let (d_soft, _) = solve(1e-3);
        assert!(d_soft > d_stiff, "soft {d_soft} should stay further from rest than stiff {d_stiff}");
        assert!(d_soft <= 2.0);
    }

    #[test]
    fn free_fall_matches_semi_implicit_euler() {
        let mut p = Particle::new([0.0, 0.0, 0.0], 1.0);
        let h = 0.01;
        for _ in 0..100 {
            p.prev = p.pos;
            p.vel = add(p.vel, scale([0.0, -9.81, 0.0], h));
            p.pos = add(p.pos, scale(p.vel, h));
        }
        // x_N = g·t·(t+h)/2 for semi-implicit Euler
        let expected = -9.81 * 1.0 * (1.0 + h) / 2.0;
        assert!((p.pos[1] - expected).abs() < 1e-9, "{} vs {}", p.pos[1], expected);
    }

    #[test]
    fn hair_strand_is_stable_and_stays_attached() {
        let mut s = HairStrand::new([0.0, 0.0, 0.0], 7, 0.35, 1e-7, 1e-6);
        for _ in 0..400 {
            s.step(1.0 / 60.0, 4, 8);
        }
        for p in &s.particles {
            assert!(p.pos.iter().all(|x| x.is_finite()), "non-finite position");
        }
        assert!((s.particles[0].pos[1] - 0.0).abs() < 1e-12, "root moved");
        assert!(s.tip()[1] < 0.0, "tip should hang below the root");
        assert!(s.tip()[1] > -10.0, "tip exploded: {}", s.tip()[1]);
        assert!(s.max_stretch() < 0.2, "over-stretched: {}", s.max_stretch());
    }

    #[test]
    fn bend_link_straightens_an_l_shape() {
        // three particles at a right angle, with a bend link across the middle
        let mut ps = [
            Particle::new([0.0, 0.0, 0.0], 1.0),
            Particle::new([1.0, 0.0, 0.0], 1.0),
            Particle::new([1.0, 1.0, 0.0], 1.0),
        ];
        let mut bend = DistanceConstraint::new(0, 2, 2.0, 0.0);
        // the endpoints start sqrt(2) apart, rest is 2 -> C < 0
        let (big_c, _) = solve_distance_xpbd(&mut bend, &mut ps, 1.0 / 60.0);
        assert!(big_c < 0.0);
        let after = len(sub(ps[0].pos, ps[2].pos));
        assert!(after > 2.0f64.sqrt(), "bend link should pull the chain straighter: {after}");
    }
}
