//! Deterministic raster renderer built on tiny-skia.
//!
//! A stroke is rendered as a chain of overlapping brush stamps whose radius and
//! alpha follow the recorded pressure. That is the same model a graphics tablet
//! driver uses, and it means the renderer is model-agnostic: any component that
//! emits `Stroke`s gets identical output.

use buncraft_core::document::{
    Action, Document, FillRegion, LAYER_BACKGROUND, LAYER_CONSTRUCTION,
};
use buncraft_core::geom::{self, Vec2};
use buncraft_core::rng::noise2;
use buncraft_core::stroke::{BrushKind, Rgba, Stage, Stroke};
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Point,
    RadialGradient, Shader, SpreadMode, Transform,
};

#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// Output scale relative to the document resolution.
    pub scale: f32,
    /// Draw only actions that have started by this time (replay).
    pub up_to_ms: Option<f64>,
    /// Hide construction lines (useful for the final artwork).
    pub hide_construction: bool,
    /// Override the canvas background.
    pub background: Option<Rgba>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            up_to_ms: None,
            hide_construction: false,
            background: None,
        }
    }
}

impl RenderOptions {
    pub fn final_image() -> Self {
        Self {
            hide_construction: true,
            ..Default::default()
        }
    }

    pub fn at_time(ms: f64) -> Self {
        Self {
            up_to_ms: Some(ms),
            ..Default::default()
        }
    }
}

fn to_color(c: Rgba, alpha_mul: f32) -> Color {
    let a = (c.a as f32 / 255.0) * alpha_mul.clamp(0.0, 1.0);
    Color::from_rgba8(c.r, c.g, c.b, (a * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// Render the whole document.
pub fn render(doc: &Document, opts: &RenderOptions) -> Pixmap {
    let scale = opts.scale.max(0.05);
    let w = ((doc.canvas.width as f32 * scale).round() as u32).max(1);
    let h = ((doc.canvas.height as f32 * scale).round() as u32).max(1);
    let mut out = Pixmap::new(w, h).expect("valid pixmap size");
    let bg = opts.background.unwrap_or(doc.canvas.background);
    out.fill(to_color(bg, 1.0));

    let mut layers: Vec<&buncraft_core::Layer> = doc.layers.iter().filter(|l| l.visible).collect();
    layers.sort_by_key(|l| l.id);

    for layer in layers {
        if opts.hide_construction && layer.id == LAYER_CONSTRUCTION {
            continue;
        }
        let mut lp = Pixmap::new(w, h).expect("valid pixmap size");
        let mut drew = false;
        for (i, action) in doc.actions.iter().enumerate() {
            if action.layer() != layer.id {
                continue;
            }
            let progress = match (opts.up_to_ms, doc.timings.get(i)) {
                (Some(t), Some(timing)) => timing.progress(t),
                (Some(_), None) => 0.0,
                (None, _) => 1.0,
            };
            if progress <= 0.0 {
                continue;
            }
            draw_action(&mut lp, action, progress, scale, doc.meta.seed);
            drew = true;
        }
        if !drew {
            continue;
        }
        let paint = PixmapPaint {
            opacity: layer.opacity.clamp(0.0, 1.0),
            blend_mode: BlendMode::SourceOver,
            quality: tiny_skia::FilterQuality::Nearest,
        };
        out.draw_pixmap(0, 0, lp.as_ref(), &paint, Transform::identity(), None);
    }
    out
}

fn draw_action(pm: &mut Pixmap, action: &Action, progress: f32, scale: f32, seed: u64) {
    match action {
        Action::Fill(f) => draw_fill(pm, f, progress, scale),
        Action::Stroke(s) => {
            if let Some(clip) = &s.clip {
                let (w, h) = (pm.width(), pm.height());
                let mut tmp = Pixmap::new(w, h).expect("valid pixmap");
                draw_stroke(&mut tmp, s, progress, scale, seed);
                if let Some(mask) = polygon_mask(clip, w, h, scale) {
                    tmp.apply_mask(&mask);
                }
                pm.draw_pixmap(
                    0,
                    0,
                    tmp.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            } else {
                draw_stroke(pm, s, progress, scale, seed);
            }
        }
    }
}

fn draw_fill(pm: &mut Pixmap, f: &FillRegion, progress: f32, scale: f32) {
    if f.polygon.len() < 3 {
        return;
    }
    // Fills are revealed as a wipe so replay still looks like painting.
    let mut path = PathBuilder::new();
    let p0 = f.polygon[0];
    path.move_to(p0.x * scale, p0.y * scale);
    for p in &f.polygon[1..] {
        path.line_to(p.x * scale, p.y * scale);
    }
    path.close();
    let Some(path) = path.finish() else { return };

    if let Some(clip) = &f.clip {
        let (w, h) = (pm.width(), pm.height());
        let mut tmp = Pixmap::new(w, h).expect("valid pixmap");
        let mut paint = Paint::default();
        paint.shader = Shader::SolidColor(to_color(f.color, f.opacity * progress));
        paint.anti_alias = true;
        tmp.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        if let Some(mask) = polygon_mask(clip, w, h, scale) {
            tmp.apply_mask(&mask);
        }
        pm.draw_pixmap(
            0,
            0,
            tmp.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        return;
    }

    let mut paint = Paint::default();
    paint.shader = Shader::SolidColor(to_color(f.color, f.opacity * progress));
    paint.anti_alias = true;
    pm.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

fn polygon_mask(clip: &[Vec2], w: u32, h: u32, scale: f32) -> Option<Mask> {
    if clip.len() < 3 {
        return None;
    }
    let mut path = PathBuilder::new();
    path.move_to(clip[0].x * scale, clip[0].y * scale);
    for p in &clip[1..] {
        path.line_to(p.x * scale, p.y * scale);
    }
    path.close();
    let path = path.finish()?;
    let mut mask = Mask::new(w, h)?;
    mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
    Some(mask)
}

fn draw_stroke(pm: &mut Pixmap, s: &Stroke, progress: f32, scale: f32, doc_seed: u64) {
    let n = s.points.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        stamp(pm, s.points[0].pos(), s, s.points[0].pressure, scale, doc_seed);
        return;
    }
    let visible = ((n - 1) as f32 * progress).max(0.0);
    let whole = visible.floor() as usize;
    let frac = visible - whole as f32;
    let last_index = whole.min(n - 1);

    for i in 0..last_index {
        segment(pm, s, i, 1.0, scale, doc_seed);
    }
    if last_index + 1 < n && frac > 0.0 {
        segment(pm, s, last_index, frac, scale, doc_seed);
    } else if whole >= n - 1 {
        // ensure the final stamp is present
        let p = s.points[n - 1];
        stamp(pm, p.pos(), s, p.pressure, scale, doc_seed);
    }
}

fn segment(pm: &mut Pixmap, s: &Stroke, i: usize, frac: f32, scale: f32, doc_seed: u64) {
    let a = s.points[i];
    let b = s.points[i + 1];
    let pa = a.pos();
    let pb = b.pos();
    let d = pa.dist(pb);
    if d < 1e-5 {
        return;
    }
    let radius = (s.brush.size * 0.5).max(0.35);
    let spacing = (radius * 0.55).clamp(0.5, 3.0);
    let steps = ((d / spacing).ceil() as usize).clamp(1, 600);
    for k in 0..=steps {
        let t = (k as f32 / steps as f32) * frac;
        if t > 1.0 {
            break;
        }
        let p = pa.lerp(pb, t);
        let pressure = a.pressure + (b.pressure - a.pressure) * t;
        stamp(pm, p, s, pressure, scale, doc_seed);
    }
}

fn stamp(pm: &mut Pixmap, p: Vec2, s: &Stroke, pressure: f32, scale: f32, doc_seed: u64) {
    let pressure = pressure.clamp(0.03, 1.0);
    let base_r = s.brush.size * 0.5;
    let seed = s.seed ^ doc_seed.rotate_left(17);

    // hand tremor: a small stable offset per stamp position
    let jx = if s.brush.jitter > 0.0 {
        (noise2(p.x * 0.35, p.y * 0.35, seed) - 0.5) * s.brush.jitter * 2.0
    } else {
        0.0
    };
    let jy = if s.brush.jitter > 0.0 {
        (noise2(p.y * 0.35 + 11.0, p.x * 0.35 + 7.0, seed) - 0.5) * s.brush.jitter * 2.0
    } else {
        0.0
    };
    let x = (p.x + jx) * scale;
    let y = (p.y + jy) * scale;
    let r = (base_r * scale).max(0.25);

    let color = s.brush.color;
    match s.brush.kind {
        BrushKind::Pencil => {
            let grain = 0.45 + 0.55 * noise2(p.x * 0.9, p.y * 0.9, seed ^ 0x51);
            let alpha = s.brush.opacity * pressure.powf(0.75) * grain;
            let rr = r * (0.55 + 0.5 * pressure);
            circle(pm, x, y, rr, color, alpha);
            if grain > 0.72 {
                circle(pm, x + r * 0.45, y - r * 0.35, rr * 0.5, color, alpha * 0.6);
            }
        }
        BrushKind::Ink => {
            let alpha = s.brush.opacity * (0.82 + 0.18 * pressure);
            let rr = r * (0.72 + 0.32 * pressure);
            circle(pm, x, y, rr, color, alpha);
        }
        BrushKind::Brush => {
            let alpha = s.brush.opacity * (0.55 + 0.45 * pressure);
            let rr = r * (0.35 + 0.8 * pressure);
            circle(pm, x, y, rr, color, alpha);
        }
        BrushKind::Marker => {
            let alpha = s.brush.opacity * (0.55 + 0.45 * pressure);
            circle(pm, x, y, r * 1.05, color, alpha);
        }
        BrushKind::Airbrush => {
            let alpha = s.brush.opacity * (0.35 + 0.65 * pressure);
            soft_circle(pm, x, y, r * 1.6, color, alpha);
        }
    }
}

/// Fast premultiplied-alpha disc. Handles the small radii that make up almost
/// every stamp in a drawing, where building a path per stamp would dominate the
/// frame time.
fn circle(pm: &mut Pixmap, x: f32, y: f32, r: f32, color: Rgba, alpha: f32) {
    if alpha <= 0.002 || r <= 0.05 {
        return;
    }
    if r > 26.0 {
        circle_path(pm, x, y, r, color, alpha);
        return;
    }
    let w = pm.width() as i32;
    let h = pm.height() as i32;
    let x0 = ((x - r - 1.0).floor() as i32).clamp(0, w);
    let x1 = ((x + r + 1.0).ceil() as i32).clamp(0, w);
    let y0 = ((y - r - 1.0).floor() as i32).clamp(0, h);
    let y1 = ((y + r + 1.0).ceil() as i32).clamp(0, h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let a0 = alpha.clamp(0.0, 1.0);
    let (cr, cg, cb) = (color.r as f32, color.g as f32, color.b as f32);
    let data = pm.data_mut();
    for py in y0..y1 {
        let dy = py as f32 + 0.5 - y;
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - x;
            let d2 = dx * dx + dy * dy;
            if d2 > (r + 0.5) * (r + 0.5) {
                continue;
            }
            let d = d2.sqrt();
            let cov = (r + 0.5 - d).clamp(0.0, 1.0);
            let a = a0 * cov;
            if a <= 0.0005 {
                continue;
            }
            let idx = ((py * w + px) * 4) as usize;
            let inv = 1.0 - a;
            data[idx] = (cr * a + data[idx] as f32 * inv + 0.5) as u8;
            data[idx + 1] = (cg * a + data[idx + 1] as f32 * inv + 0.5) as u8;
            data[idx + 2] = (cb * a + data[idx + 2] as f32 * inv + 0.5) as u8;
            data[idx + 3] = (255.0 * a + data[idx + 3] as f32 * inv + 0.5) as u8;
        }
    }
}

fn circle_path(pm: &mut Pixmap, x: f32, y: f32, r: f32, color: Rgba, alpha: f32) {
    let mut pb = PathBuilder::new();
    pb.push_circle(x, y, r);
    let Some(path) = pb.finish() else { return };
    let mut paint = Paint::default();
    paint.shader = Shader::SolidColor(to_color(color, alpha));
    paint.anti_alias = true;
    pm.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

/// Airbrush: one radial gradient instead of stacked discs, so the falloff is
/// smooth and a 700px background wash costs a single fill.
fn soft_circle(pm: &mut Pixmap, x: f32, y: f32, r: f32, color: Rgba, alpha: f32) {
    if alpha <= 0.002 || r <= 0.5 {
        return;
    }
    let mut pb = PathBuilder::new();
    pb.push_circle(x, y, r);
    let Some(path) = pb.finish() else { return };
    let shader = RadialGradient::new(
        Point::from_xy(x, y),
        Point::from_xy(x, y),
        r,
        vec![
            GradientStop::new(0.0, to_color(color, alpha)),
            GradientStop::new(0.55, to_color(color, alpha * 0.55)),
            GradientStop::new(1.0, to_color(color, 0.0)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    );
    let Some(shader) = shader else { return };
    let mut paint = Paint::default();
    paint.shader = shader;
    paint.anti_alias = true;
    pm.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------

/// Vector export. Every stroke becomes a cubic path; the output opens in
/// Inkscape/Illustrator and can drive a pen plotter.
pub fn to_svg(doc: &Document) -> String {
    let mut out = String::with_capacity(doc.actions.len() * 220);
    out.push_str(&format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        doc.canvas.width, doc.canvas.height, doc.canvas.width, doc.canvas.height
    ));
    out.push_str(&format!(
        "  <rect width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
        doc.canvas.width,
        doc.canvas.height,
        doc.canvas.background.hex()
    ));

    // One-point airbrush strokes are dabs. Exported as plain paths they would
    // be dropped (a path needs two points), which silently removed the blush
    // and every soft shadow from the SVG, so they become radial gradients.
    let mut defs = String::new();
    let mut dab = 0usize;

    let mut layers: Vec<&buncraft_core::Layer> = doc.layers.iter().filter(|l| l.visible).collect();
    layers.sort_by_key(|l| l.id);
    for layer in layers {
        out.push_str(&format!(
            "  <g id=\"layer-{}\" opacity=\"{:.3}\">\n",
            layer.id, layer.opacity
        ));
        for action in &doc.actions {
            if action.layer() != layer.id {
                continue;
            }
            match action {
                Action::Stroke(s) => {
                    if s.points.len() == 1 && matches!(s.brush.kind, BrushKind::Airbrush) {
                        let p = s.points[0].pos();
                        let pressure = s.points[0].pressure.clamp(0.03, 1.0);
                        let alpha = (s.brush.opacity * (0.35 + 0.65 * pressure)).clamp(0.0, 1.0);
                        let r = s.brush.size * 0.8;
                        let id = format!("dab{dab}");
                        dab += 1;
                        let hex = s.brush.color.hex();
                        defs.push_str(&format!(
                            "    <radialGradient id=\"{id}\"><stop offset=\"0\" stop-color=\"{hex}\" stop-opacity=\"{alpha:.3}\"/><stop offset=\"0.55\" stop-color=\"{hex}\" stop-opacity=\"{:.3}\"/><stop offset=\"1\" stop-color=\"{hex}\" stop-opacity=\"0\"/></radialGradient>\n",
                            alpha * 0.55
                        ));
                        out.push_str(&format!(
                            "    <circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"url(#{id})\"/>\n",
                            p.x, p.y, r
                        ));
                        continue;
                    }
                    if s.points.len() < 2 {
                        continue;
                    }
                    out.push_str(&format!(
                        "    <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-opacity=\"{:.3}\" stroke-width=\"{:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n",
                        stroke_path_d(s),
                        s.brush.color.hex(),
                        s.brush.opacity,
                        s.brush.size
                    ));
                }
                Action::Fill(f) => {
                    if f.polygon.len() < 3 {
                        continue;
                    }
                    out.push_str(&format!(
                        "    <path d=\"{}\" fill=\"{}\" fill-opacity=\"{:.3}\"/>\n",
                        polygon_d(&f.polygon),
                        f.color.hex(),
                        f.opacity
                    ));
                }
            }
        }
        out.push_str("  </g>\n");
    }
    if !defs.is_empty() {
        out.push_str("  <defs>\n");
        out.push_str(&defs);
        out.push_str("  </defs>\n");
    }
    out.push_str("</svg>\n");
    out
}

fn polygon_d(points: &[Vec2]) -> String {
    let mut d = String::new();
    for (i, p) in points.iter().enumerate() {
        if i == 0 {
            d.push_str(&format!("M {:.2} {:.2}", p.x, p.y));
        } else {
            d.push_str(&format!(" L {:.2} {:.2}", p.x, p.y));
        }
    }
    d.push_str(" Z");
    d
}

fn stroke_path_d(s: &Stroke) -> String {
    let pts: Vec<Vec2> = s.points.iter().map(|p| p.pos()).collect();
    if pts.len() < 2 {
        return String::new();
    }
    let mut d = format!("M {:.2} {:.2}", pts[0].x, pts[0].y);
    if pts.len() == 2 {
        d.push_str(&format!(" L {:.2} {:.2}", pts[1].x, pts[1].y));
        return d;
    }
    for i in 0..pts.len() - 1 {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < pts.len() {
            pts[i + 2]
        } else {
            pts[pts.len() - 1]
        };
        let c1 = p1.add(p2.sub(p0).mul(1.0 / 6.0));
        let c2 = p2.sub(p3.sub(p1).mul(1.0 / 6.0));
        d.push_str(&format!(
            " C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            c1.x, c1.y, c2.x, c2.y, p2.x, p2.y
        ));
    }
    d
}

/// Raster export helper.
pub fn save_png(
    doc: &Document,
    path: impl AsRef<std::path::Path>,
    opts: &RenderOptions,
) -> Result<(), String> {
    let pm = render(doc, opts);
    pm.save_png(path.as_ref()).map_err(|e| e.to_string())
}

/// Number of stages that actually contain actions, in order.
pub fn stages_used(doc: &Document) -> Vec<Stage> {
    let mut out = Vec::new();
    for a in &doc.actions {
        if !out.contains(&a.stage()) {
            out.push(a.stage());
        }
    }
    out
}

/// Coverage estimate used by the evaluator: fraction of a coarse grid that has
/// any action passing through it.
pub fn ink_coverage(doc: &Document, grid: u32) -> f32 {
    let grid = grid.max(8);
    let mut cells = vec![false; (grid * grid) as usize];
    let cw = doc.canvas.width as f32 / grid as f32;
    let ch = doc.canvas.height as f32 / grid as f32;
    for a in &doc.actions {
        if a.layer() == LAYER_BACKGROUND || a.layer() == LAYER_CONSTRUCTION {
            continue;
        }
        // fills are colour, not ink
        if matches!(a, Action::Fill(_)) {
            continue;
        }
        let b = a.bounds();
        if b.is_empty() {
            continue;
        }
        let x0 = (b.min.x / cw).floor().max(0.0) as u32;
        let x1 = (b.max.x / cw).ceil().min(grid as f32) as u32;
        let y0 = (b.min.y / ch).floor().max(0.0) as u32;
        let y1 = (b.max.y / ch).ceil().min(grid as f32) as u32;
        for y in y0..y1.min(grid) {
            for x in x0..x1.min(grid) {
                cells[(y * grid + x) as usize] = true;
            }
        }
    }
    let hit = cells.iter().filter(|c| **c).count();
    hit as f32 / cells.len() as f32
}

/// Distance helper re-exported for the evaluator.
pub fn polyline_distance(p: Vec2, line: &[Vec2]) -> f32 {
    geom::distance_to_polyline(p, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buncraft_core::document::{Canvas, DocMeta, LAYER_FLAT, LAYER_LINEART};
    use buncraft_core::stroke::Brush;

    fn sample_doc() -> Document {
        let mut d = Document::new(Canvas::new(200, 200), DocMeta::new("t", "p", 3));
        let b = Brush::new(BrushKind::Ink, 4.0, Rgba::rgb(0, 0, 0));
        d.stroke(
            &[Vec2::new(20.0, 100.0), Vec2::new(180.0, 100.0)],
            b,
            LAYER_LINEART,
            Stage::Lineart,
            "line",
        );
        d.schedule(1.0);
        d
    }

    #[test]
    fn svg_keeps_single_point_airbrush_dabs() {
        let mut d = Document::new(Canvas::new(200, 200), DocMeta::new("t", "p", 3));
        let dab = Brush::new(BrushKind::Airbrush, 40.0, Rgba::rgb(240, 148, 160));
        d.stroke(&[Vec2::new(100.0, 100.0)], dab, LAYER_FLAT, Stage::Shading, "blush");
        let svg = to_svg(&d);
        assert!(svg.contains("<radialGradient"), "dab must keep its falloff");
        assert!(svg.contains("<circle"), "dab must be exported at all");
        assert!(svg.contains("url(#dab0)"), "circle must reference the gradient");
    }

    #[test]
    fn renders_ink_pixels() {
        let d = sample_doc();
        let pm = render(&d, &RenderOptions::default());
        assert_eq!(pm.width(), 200);
        // the middle of the line must be darker than the corner
        let mid = pm.pixel(100, 100).unwrap();
        let corner = pm.pixel(5, 5).unwrap();
        assert!(mid.alpha() > 0);
        assert!(mid.red() < corner.red());
    }

    #[test]
    fn replay_starts_blank() {
        let d = sample_doc();
        let pm = render(&d, &RenderOptions::at_time(0.0));
        let mid = pm.pixel(100, 100).unwrap();
        let corner = pm.pixel(5, 5).unwrap();
        assert_eq!(mid.red(), corner.red());
    }

    #[test]
    fn svg_contains_path() {
        let d = sample_doc();
        let svg = to_svg(&d);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("stroke-width"));
        assert!(svg.contains("M 20.00 100.00"));
    }

    #[test]
    fn clipped_stroke_stays_inside_mask() {
        let mut d = Document::new(Canvas::new(100, 100), DocMeta::new("t", "p", 1));
        let b = Brush::new(BrushKind::Marker, 10.0, Rgba::rgb(255, 0, 0));
        let id = d.stroke(
            &[Vec2::new(0.0, 50.0), Vec2::new(100.0, 50.0)],
            b,
            LAYER_FLAT,
            Stage::Color,
            "band",
        );
        let clip = vec![
            Vec2::new(20.0, 20.0),
            Vec2::new(40.0, 20.0),
            Vec2::new(40.0, 80.0),
            Vec2::new(20.0, 80.0),
        ];
        if let Some(Action::Stroke(s)) = d.actions.get_mut(id as usize - 1) {
            s.clip = Some(clip);
        }
        let pm = render(&d, &RenderOptions::default());
        let inside = pm.pixel(30, 50).unwrap();
        let outside = pm.pixel(80, 50).unwrap();
        assert!(inside.red() > 200);
        assert_eq!(outside.red(), 252);
    }
}
