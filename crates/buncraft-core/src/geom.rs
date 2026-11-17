//! 2D geometry primitives and curve helpers used by every BunCraft component.
//!
//! All coordinates are in canvas space: origin top-left, +x right, +y down,
//! unit = one pixel at the document's base resolution.

use serde::{Deserialize, Serialize};

use buncraft_science::curve::CubicBez;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }

    pub fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }

    pub fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn dist(self, o: Vec2) -> f32 {
        self.sub(o).len()
    }

    pub fn norm(self) -> Vec2 {
        let l = self.len();
        if l < 1e-6 {
            Vec2::ZERO
        } else {
            self.mul(1.0 / l)
        }
    }

    pub fn lerp(self, o: Vec2, t: f32) -> Vec2 {
        Vec2::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }

    /// Rotate 90 degrees counter-clockwise in screen space.
    pub fn perp(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }

    pub fn rot(self, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }

    pub fn dot(self, o: Vec2) -> f32 {
        self.x * o.x + self.y * o.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    pub fn empty() -> Self {
        Self {
            min: Vec2::new(f32::INFINITY, f32::INFINITY),
            max: Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn from_point(p: Vec2) -> Self {
        Self { min: p, max: p }
    }

    pub fn include(&mut self, p: Vec2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn union(&mut self, o: Bounds) {
        self.include(o.min);
        self.include(o.max);
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y
    }

    pub fn width(&self) -> f32 {
        (self.max.x - self.min.x).max(0.0)
    }

    pub fn height(&self) -> f32 {
        (self.max.y - self.min.y).max(0.0)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }
}

/// Uniform Catmull-Rom interpolation through `points`.
///
/// The curve passes through every control point, which makes it a good fit for
/// hand-drawn strokes: the artist's anchors are exactly the points the pen
/// touches, and the spline only smooths the motion between them.
pub fn catmull_rom(points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let samples = samples_per_segment.max(1);
    let n = points.len();
    let mut out = Vec::with_capacity((n - 1) * samples + 1);
    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };
        for s in 0..samples {
            let t = s as f32 / samples as f32;
            out.push(catmull_segment(p0, p1, p2, p3, t));
        }
    }
    out.push(points[n - 1]);
    out
}

/// Closed Catmull-Rom loop. The returned polyline is closed (last == first).
pub fn catmull_rom_closed(points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let samples = samples_per_segment.max(1);
    let n = points.len();
    let mut out = Vec::with_capacity(n * samples + 1);
    for i in 0..n {
        let p0 = points[(i + n - 1) % n];
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];
        for s in 0..samples {
            let t = s as f32 / samples as f32;
            out.push(catmull_segment(p0, p1, p2, p3, t));
        }
    }
    out.push(out[0]);
    out
}

/// Build the cubic Béziers equivalent to a uniform Catmull–Rom chain.
///
/// For anchors `p0, p1, p2, p3` the segment from `p1` to `p2` has control
/// points `b0 = p1`, `b1 = p1 + (p2-p0)/6`, `b2 = p2 - (p3-p1)/6`, `b3 = p2`.
/// This is an exact identity, not an approximation.
fn cr_cubics(points: &[Vec2], closed: bool) -> Vec<CubicBez> {
    let n = points.len();
    let count = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let (p0, p1, p2, p3) = if closed {
            (
                points[(i + n - 1) % n],
                points[i],
                points[(i + 1) % n],
                points[(i + 2) % n],
            )
        } else {
            let p0 = if i == 0 { points[0] } else { points[i - 1] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };
            (p0, p1, p2, p3)
        };
        let b1 = Vec2::new(
            p1.x + (p2.x - p0.x) / 6.0,
            p1.y + (p2.y - p0.y) / 6.0,
        );
        let b2 = Vec2::new(
            p2.x - (p3.x - p1.x) / 6.0,
            p2.y - (p3.y - p1.y) / 6.0,
        );
        out.push(CubicBez::new(
            [p1.x as f64, p1.y as f64],
            [b1.x as f64, b1.y as f64],
            [b2.x as f64, b2.y as f64],
            [p2.x as f64, p2.y as f64],
        ));
    }
    out
}

/// True arc length of every segment, plus the cumulative table.
fn cubic_lengths(segs: &[CubicBez]) -> (Vec<f64>, Vec<f64>) {
    let mut lens = Vec::with_capacity(segs.len());
    let mut cum = Vec::with_capacity(segs.len() + 1);
    cum.push(0.0);
    for s in segs {
        let l = s.arclen(0.02);
        lens.push(l);
        cum.push(cum.last().copied().unwrap_or(0.0) + l);
    }
    (lens, cum)
}

/// Walk the chain and emit `count` points spaced evenly in arc length.
fn sample_cubics(
    segs: &[CubicBez],
    lens: &[f64],
    cum: &[f64],
    count: usize,
    closed: bool,
    points: &[Vec2],
) -> Vec<Vec2> {
    let total = cum.last().copied().unwrap_or(0.0);
    if count < 2 || total < 1e-4 || segs.is_empty() {
        return points.to_vec();
    }
    let spacing = total / (count - 1) as f64;
    let mut out = Vec::with_capacity(count);
    let mut si = 0usize;
    for i in 0..count {
        let s = (i as f64 * spacing).min(total);
        while si + 1 < segs.len() && cum[si + 1] < s {
            si += 1;
        }
        let local = (s - cum[si]).clamp(0.0, lens[si]);
        let t = segs[si].inv_arclen_with_total(local, lens[si], 0.02);
        let p = segs[si].eval(t);
        out.push(Vec2::new(p[0] as f32, p[1] as f32));
    }
    if closed {
        // A closed loop repeats its first point so the renderer can close it.
        out[count - 1] = out[0];
    } else {
        out[0] = points[0];
        out[count - 1] = points[points.len() - 1];
    }
    out
}

/// Catmull–Rom sampled evenly by **arc length** instead of by the curve
/// parameter. Same anchors and roughly the same point count as
/// [`catmull_rom`], but a long segment now receives proportionally more
/// samples than a short one, which is what makes pen speed physical.
pub fn catmull_rom_arclen(
    points: &[Vec2],
    samples_per_segment: usize,
    closed: bool,
) -> Vec<Vec2> {
    let n = points.len();
    if n < 2 || (closed && n < 3) {
        return points.to_vec();
    }
    let segs = cr_cubics(points, closed);
    let (lens, cum) = cubic_lengths(&segs);
    let count = segs.len() * samples_per_segment.max(1) + 1;
    sample_cubics(&segs, &lens, &cum, count, closed, points)
}

/// Catmull–Rom sampled at a fixed arc-length `spacing` (in pixels).
/// This is what the stroke builder wants: stamps at a constant distance,
/// independent of how the anchors were distributed.
pub fn catmull_rom_arclen_spacing(points: &[Vec2], spacing: f32, closed: bool) -> Vec<Vec2> {
    let n = points.len();
    if n < 2 || (closed && n < 3) || spacing <= 0.01 {
        return points.to_vec();
    }
    let segs = cr_cubics(points, closed);
    let (lens, cum) = cubic_lengths(&segs);
    let total = cum.last().copied().unwrap_or(0.0);
    if total < 1e-4 {
        return points.to_vec();
    }
    let count = ((total / spacing as f64).round() as usize).clamp(1, 200_000) + 1;
    sample_cubics(&segs, &lens, &cum, count, closed, points)
}

pub fn catmull_segment(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let x = 0.5
        * ((2.0 * p1.x)
            + (-p0.x + p2.x) * t
            + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
            + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);
    let y = 0.5
        * ((2.0 * p1.y)
            + (-p0.y + p2.y) * t
            + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
            + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);
    Vec2::new(x, y)
}

/// Resample a polyline so consecutive points are roughly `spacing` apart.
/// The first and last points are preserved exactly.
pub fn resample(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if points.len() < 2 || spacing <= 0.01 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    let mut carry = 0.0f32;
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = a.dist(b);
        if seg <= 1e-6 {
            continue;
        }
        let mut d = spacing - carry;
        while d < seg {
            out.push(a.lerp(b, d / seg));
            d += spacing;
        }
        carry = seg - (d - spacing);
        carry %= spacing;
    }
    if let Some(last) = points.last() {
        if out.last().map(|p| p.dist(*last)).unwrap_or(0.0) > spacing * 0.25 {
            out.push(*last);
        } else if let Some(slot) = out.last_mut() {
            *slot = *last;
        }
    }
    out
}

/// Signed area of a polygon (positive = clockwise in screen coordinates).
pub fn polygon_area(points: &[Vec2]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        sum += a.x * b.y - b.x * a.y;
    }
    sum * 0.5
}

pub fn polygon_centroid(points: &[Vec2]) -> Vec2 {
    let area = polygon_area(points);
    if area.abs() < 1e-6 {
        let mut s = Vec2::ZERO;
        for p in points {
            s = s.add(*p);
        }
        return s.mul(1.0 / points.len().max(1) as f32);
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let cross = a.x * b.y - b.x * a.y;
        cx += (a.x + b.x) * cross;
        cy += (a.y + b.y) * cross;
    }
    Vec2::new(cx / (6.0 * area), cy / (6.0 * area))
}

/// Mirror points across a vertical axis.
pub fn mirror_x(points: &[Vec2], axis: f32) -> Vec<Vec2> {
    points.iter().map(|p| Vec2::new(2.0 * axis - p.x, p.y)).collect()
}

/// Point on an ellipse, angle in radians (0 = right, PI/2 = down).
pub fn ellipse_point(center: Vec2, rx: f32, ry: f32, angle: f32) -> Vec2 {
    Vec2::new(
        center.x + rx * angle.cos(),
        center.y + ry * angle.sin(),
    )
}

/// Sample a full ellipse as a closed polyline.
pub fn ellipse(center: Vec2, rx: f32, ry: f32, samples: usize) -> Vec<Vec2> {
    let n = samples.max(8);
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        pts.push(ellipse_point(center, rx, ry, a));
    }
    pts
}

/// Sample an elliptical arc from `start` to `end` radians (inclusive).
pub fn arc(
    center: Vec2,
    rx: f32,
    ry: f32,
    start: f32,
    end: f32,
    samples: usize,
) -> Vec<Vec2> {
    let n = samples.max(2);
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let a = start + (end - start) * t;
        pts.push(ellipse_point(center, rx, ry, a));
    }
    pts
}

/// Approximate distance from a point to a polyline.
pub fn distance_to_polyline(p: Vec2, line: &[Vec2]) -> f32 {
    if line.is_empty() {
        return f32::INFINITY;
    }
    if line.len() == 1 {
        return p.dist(line[0]);
    }
    let mut best = f32::INFINITY;
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b.sub(a);
        let len2 = ab.dot(ab);
        let t = if len2 < 1e-9 {
            0.0
        } else {
            (p.sub(a).dot(ab) / len2).clamp(0.0, 1.0)
        };
        let q = a.add(ab.mul(t));
        best = best.min(p.dist(q));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_passes_through_anchors() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 20.0),
            Vec2::new(30.0, 0.0),
        ];
        let curve = catmull_rom(&pts, 8);
        assert!(curve[0].dist(pts[0]) < 1e-5);
        assert!(curve.last().unwrap().dist(pts[2]) < 1e-5);
        // the middle anchor must appear at the segment boundary
        assert!(curve[8].dist(pts[1]) < 1e-4);
    }

    #[test]
    fn closed_curve_is_closed() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(5.0, 10.0),
        ];
        let curve = catmull_rom_closed(&pts, 6);
        assert!(curve.first().unwrap().dist(*curve.last().unwrap()) < 1e-4);
    }

    #[test]
    fn resample_keeps_endpoints() {
        let pts = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)];
        let out = resample(&pts, 10.0);
        assert_eq!(out.first().unwrap(), &pts[0]);
        assert_eq!(out.last().unwrap(), &pts[1]);
        assert!(out.len() >= 10);
    }

    #[test]
    fn polygon_area_of_square() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        assert!((polygon_area(&pts).abs() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn arclen_sampling_is_even_on_uneven_anchors() {
        // Four points on a circle at very uneven angular spacing: a 5° hop,
        // a 115° sweep, then another 5° hop. Uniform-t sampling gives the
        // short segments the same number of points as the long one.
        let pts: Vec<Vec2> = [0.0f32, 20.0, 70.0, 90.0]
            .iter()
            .map(|deg| {
                let a = deg.to_radians();
                Vec2::new(100.0 * a.cos(), 100.0 * a.sin())
            })
            .collect();
        let even = catmull_rom_arclen(&pts, 20, false);
        let dists: Vec<f32> = even.windows(2).map(|w| w[0].dist(w[1])).collect();
        let max = dists.iter().cloned().fold(0.0f32, f32::max);
        let min = dists.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(max / min < 1.02, "arc-length ratio {}", max / min);

        // The uniform-t sampler is far worse on the same anchors.
        let uniform = catmull_rom(&pts, 20);
        let ud: Vec<f32> = uniform.windows(2).map(|w| w[0].dist(w[1])).collect();
        let umax = ud.iter().cloned().fold(0.0f32, f32::max);
        let umin = ud.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(umax / umin > 2.0, "uniform ratio {}", umax / umin);
    }

    #[test]
    fn arclen_sampling_keeps_endpoints_and_anchors() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 30.0),
            Vec2::new(100.0, 0.0),
        ];
        let out = catmull_rom_arclen(&pts, 12, false);
        assert!(out.first().unwrap().dist(pts[0]) < 1e-4);
        assert!(out.last().unwrap().dist(pts[2]) < 1e-4);
        // Arc-length sampling does not have to land *on* an anchor: the
        // anchor can fall between two samples. It must still pass within one
        // sample spacing of it.
        let mid = out
            .iter()
            .map(|p| p.dist(pts[1]))
            .fold(f32::INFINITY, f32::min);
        let spacing = out.windows(2).map(|w| w[0].dist(w[1])).sum::<f32>()
            / (out.len() - 1) as f32;
        assert!(mid < spacing, "closest approach {mid} > spacing {spacing}");
    }

    #[test]
    fn arclen_closed_curve_closes() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(60.0, 0.0),
            Vec2::new(60.0, 60.0),
            Vec2::new(0.0, 60.0),
        ];
        let out = catmull_rom_arclen(&pts, 10, true);
        assert!(out.first().unwrap().dist(*out.last().unwrap()) < 1e-4);
        // and the spacing is even around the loop
        let dists: Vec<f32> = out.windows(2).map(|w| w[0].dist(w[1])).collect();
        let max = dists.iter().cloned().fold(0.0f32, f32::max);
        let min = dists.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(max / min < 1.05, "closed ratio {}", max / min);
    }

    #[test]
    fn arclen_spacing_hits_the_requested_distance() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(120.0, 40.0),
            Vec2::new(200.0, 0.0),
        ];
        let out = catmull_rom_arclen_spacing(&pts, 4.0, false);
        let dists: Vec<f32> = out.windows(2).map(|w| w[0].dist(w[1])).collect();
        let mean = dists.iter().sum::<f32>() / dists.len() as f32;
        assert!((mean - 4.0).abs() < 0.15, "mean spacing {mean}");
    }
}
