//! End-to-end tests: prompt -> document -> pixels -> SVG -> JSON -> reload.

use buncraft_brain::{dataset, ArtistBrain};
use buncraft_core::document::Document;
use buncraft_render::{render, to_svg, RenderOptions};

#[test]
fn draws_renders_and_reloads() {
    let brain = ArtistBrain::new();
    let out = brain.draw("silver hair, red eyes, shy smile", 7);

    // structure
    assert!(out.report.strokes > 100);
    assert!(out.report.fills >= 10);
    assert!(out.report.stages.len() >= 10);
    assert_eq!(out.document.timings.len(), out.document.actions.len());

    // pixels at quarter scale (fast in debug)
    let opts = RenderOptions {
        scale: 0.25,
        ..RenderOptions::final_image()
    };
    let pm = render(&out.document, &opts);
    assert_eq!(pm.width(), 300);
    let data = pm.data();
    let mut non_bg = 0;
    for px in data.chunks_exact(4) {
        if px[0] < 240 || px[1] < 240 || px[2] < 240 {
            non_bg += 1;
        }
    }
    assert!(
        non_bg > 2000,
        "expected visible ink, got {non_bg} dark pixels"
    );

    // replay at t=0 must be blank
    let blank = render(&out.document, &RenderOptions::at_time(0.0));
    let mid = blank.pixel(600, 800).unwrap();
    assert!(mid.red() > 240);

    // svg
    let svg = to_svg(&out.document);
    assert!(svg.starts_with("<?xml"));
    assert!(svg.contains("</svg>"));
    // Every action has to survive the vector export: strokes and fills become
    // paths, one-point airbrush dabs become gradient circles. Only genuinely
    // degenerate actions (a fill with <3 points) may be dropped.
    let drawn = svg.matches("<path").count() + svg.matches("<circle").count();
    assert!(
        drawn + 4 >= out.document.actions.len(),
        "{drawn} svg elements, {} actions",
        out.document.actions.len()
    );

    // json roundtrip preserves the drawing
    let json = out.document.to_json();
    let back: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(back.actions.len(), out.document.actions.len());
    assert_eq!(back.stroke_count(), out.document.stroke_count());
    assert_eq!(back.total_ms(), out.document.total_ms());
}

#[test]
fn same_seed_same_art() {
    let a = ArtistBrain::new().draw("black hair, sharp eyes, serious", 42);
    let b = ArtistBrain::new().draw("black hair, sharp eyes, serious", 42);
    assert_eq!(a.document.to_json(), b.document.to_json());
    assert_eq!(a.report.evaluation.strokes, b.report.evaluation.strokes);
}

#[test]
fn different_seed_changes_the_hand_not_the_character() {
    let a = ArtistBrain::new().draw("blonde ponytail, green eyes", 1);
    let b = ArtistBrain::new().draw("blonde ponytail, green eyes", 2);
    assert_eq!(a.report.character, b.report.character);
    assert_ne!(a.document.to_json(), b.document.to_json());
    // proportions must stay in the same class
    let da = (a.report.evaluation.proportion_error
        - b.report.evaluation.proportion_error)
        .abs();
    assert!(da < 0.05);
}

#[test]
fn correction_pass_only_when_needed() {
    let mut brain = ArtistBrain::new();
    brain.self_correct = false;
    let plain = brain.draw("pink twin tails, blue eyes", 3);
    let mut brain2 = ArtistBrain::new();
    brain2.self_correct = true;
    let fixed = brain2.draw("pink twin tails, blue eyes", 3);
    assert!(fixed.report.evaluation.corrections <= 4);
    assert!(plain.report.evaluation.corrections == 0);
}

#[test]
fn dataset_roundtrip_into_stroke5() {
    let dir = std::env::temp_dir().join("buncraft-e2e-dataset.jsonl");
    std::fs::write(
        &dir,
        r#"{"label":"face","strokes":[[[0,0],[10,0],[20,10]],[[30,30],[30,50]]]}
{"label":"eye","strokes":[[[5,5],[15,5]]]}
"#,
    )
    .unwrap();
    let samples = dataset::load_jsonl(&dir).unwrap();
    assert_eq!(samples.len(), 2);
    let s5 = dataset::to_stroke5(&samples[0]);
    assert_eq!(s5.point_count, 5);
    assert_eq!(s5.stroke_count, 2);
    assert_eq!(s5.points.last().unwrap()[4], 1.0); // sketch end
    let (train, val) = dataset::split(samples, 0.5);
    assert_eq!((train.len(), val.len()), (1, 1));
}
