//! BunCraft CLI: draw, inspect, benchmark.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use buncraft_brain::ArtistBrain;
use buncraft_core::document::Document;
use buncraft_render::{render, save_png, to_svg, RenderOptions};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "buncraft",
    version,
    about = "An AI that doesn't just generate art. It learns to draw."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Draw a character from a text description.
    Draw {
        /// e.g. "silver haired girl with red eyes, shy smile"
        #[arg(short, long, default_value = "")]
        prompt: String,
        #[arg(short, long, default_value_t = 1)]
        seed: u64,
        /// Output directory.
        #[arg(short, long, default_value = "assets")]
        out: PathBuf,
        /// Timeline speed multiplier (2.0 = timelapse).
        #[arg(long, default_value_t = 1.0)]
        speed: f64,
        /// Disable the self-correction pass.
        #[arg(long)]
        no_corrections: bool,
        /// Also write N evenly spaced PNG frames for video assembly.
        #[arg(long)]
        frames: Option<u32>,
    },
    /// Print the structure of a timeline.json.
    Inspect {
        #[arg(default_value = "assets/timeline.json")]
        file: PathBuf,
    },
    /// Measure generation and rendering speed.
    Bench {
        #[arg(long, default_value_t = 5)]
        runs: u32,
    },
    /// List the built-in stage pipeline.
    Stages,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Draw {
            prompt,
            seed,
            out,
            speed,
            no_corrections,
            frames,
        } => draw_cmd(&prompt, seed, &out, speed, !no_corrections, frames),
        Cmd::Inspect { file } => inspect_cmd(&file),
        Cmd::Bench { runs } => bench_cmd(runs),
        Cmd::Stages => {
            for (i, s) in [
                "composition",
                "rough sketch",
                "construction lines",
                "facial structure",
                "eyes",
                "nose & mouth",
                "hair",
                "body & anatomy",
                "clothing",
                "clean lineart",
                "colouring",
                "shadows & lighting",
                "inspection & corrections",
            ]
            .iter()
            .enumerate()
            {
                println!("{:>2}. {}", i + 1, s);
            }
            Ok(())
        }
    }
}

fn slug(prompt: &str, seed: u64) -> String {
    let mut s: String = prompt
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        format!("artwork-{seed}")
    } else {
        s.chars().take(48).collect()
    }
}

fn draw_cmd(
    prompt: &str,
    seed: u64,
    out: &Path,
    speed: f64,
    self_correct: bool,
    frames: Option<u32>,
) -> Result<()> {
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let mut brain = ArtistBrain::new();
    brain.self_correct = self_correct;
    brain.speed = speed;

    let t0 = Instant::now();
    let outcome = brain.draw(prompt, seed);
    let draw_ms = t0.elapsed().as_millis();

    let name = slug(prompt, seed);
    let doc = &outcome.document;

    let png = out.join(format!("{name}.png"));
    let svg = out.join(format!("{name}.svg"));
    let timeline = out.join("timeline.json");
    let report = out.join("report.json");
    let preview = out.join("preview.html");

    let t1 = Instant::now();
    save_png(doc, &png, &RenderOptions::final_image()).map_err(|e| anyhow::anyhow!(e))?;
    let render_ms = t1.elapsed().as_millis();

    std::fs::write(&svg, to_svg(doc))?;
    let json = doc.to_json_pretty();
    std::fs::write(&timeline, &json)?;
    // per-artwork copy, so a gallery can replay any drawing, not just the last
    std::fs::write(out.join(format!("{name}-timeline.json")), &json)?;
    std::fs::write(&report, serde_json::to_string_pretty(&outcome.report)?)?;
    std::fs::write(&preview, PLAYER_HTML)?;

    if let Some(n) = frames {
        let dir = out.join("frames");
        std::fs::create_dir_all(&dir)?;
        let n = n.max(2);
        for i in 0..n {
            let t = doc.total_ms() * i as f64 / (n - 1) as f64;
            let pm = render(doc, &RenderOptions::at_time(t));
            pm.save_png(dir.join(format!("frame_{i:04}.png")))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    }

    let e = &outcome.report.evaluation;
    println!("BunCraft — {}", outcome.report.title);
    println!(
        "  drew in {draw_ms} ms · rendered in {render_ms} ms · timeline {:.1}s",
        doc.total_ms() / 1000.0
    );
    println!(
        "  {} strokes · {} fills · {} points · {} stages",
        outcome.report.strokes,
        outcome.report.fills,
        outcome.report.points,
        outcome.report.stages.len()
    );
    if e.corrections > 0 {
        println!(
            "  self-check {:.1}/100 · symmetry {:.2}px -> {:.2}px · proportion {:.1}% -> {:.1}% · coverage {:.1}% · {} correction(s)",
            e.score,
            e.symmetry_error_before,
            e.symmetry_error,
            e.proportion_error_before * 100.0,
            e.proportion_error * 100.0,
            e.coverage * 100.0,
            e.corrections
        );
    } else {
        println!(
            "  self-check {:.1}/100 · symmetry {:.2}px · proportion {:.1}% · coverage {:.1}% · no corrections",
            e.score,
            e.symmetry_error,
            e.proportion_error * 100.0,
            e.coverage * 100.0
        );
    }
    if !e.issues.is_empty() {
        for issue in &e.issues {
            println!(
                "    - {}: {} ({})",
                issue.kind,
                issue.detail,
                if issue.corrected { "corrected" } else { "open" }
            );
        }
    }
    println!("  files:");
    for p in [&png, &svg, &timeline, &report, &preview] {
        println!("    {}", p.display());
    }
    Ok(())
}

fn inspect_cmd(file: &Path) -> Result<()> {
    let text = std::fs::read_to_string(file)
        .with_context(|| format!("reading {}", file.display()))?;
    let doc: Document = serde_json::from_str(&text).context("parsing timeline")?;
    println!("{}", doc.meta.title);
    println!("prompt: {}", doc.meta.prompt);
    println!(
        "canvas {}x{} · {} actions · {:.2}s",
        doc.canvas.width,
        doc.canvas.height,
        doc.actions.len(),
        doc.total_ms() / 1000.0
    );
    println!(
        "strokes {} · fills {} · points {}",
        doc.stroke_count(),
        doc.fill_count(),
        doc.point_count()
    );
    println!("\n{:<22} {:>7} {:>10} {:>10}", "stage", "actions", "start", "end");
    let marks = doc.stage_marks();
    for (i, (stage, start)) in marks.iter().enumerate() {
        let end = marks
            .get(i + 1)
            .map(|(_, s)| *s)
            .unwrap_or(doc.total_ms());
        let count = doc.actions.iter().filter(|a| a.stage() == *stage).count();
        println!(
            "{:<22} {:>7} {:>9.2}s {:>9.2}s",
            stage.label(),
            count,
            start / 1000.0,
            end / 1000.0
        );
    }
    if let Some(e) = &doc.evaluation {
        println!(
            "\nscore {:.1}/100 · symmetry {:.2}px · proportion {:.1}%",
            e.score,
            e.symmetry_error,
            e.proportion_error * 100.0
        );
    }
    Ok(())
}

fn bench_cmd(runs: u32) -> Result<()> {
    let prompts = [
        "silver hair, red eyes, shy smile",
        "twin tails, pink hair, blue eyes, cheerful",
        "black hair, sharp eyes, serious, coat",
        "blonde ponytail, green eyes, smug, hoodie",
        "white hair, violet eyes, surprised, kimono",
    ];
    let mut total_draw = 0.0;
    let mut total_render = 0.0;
    let mut total_strokes = 0usize;
    let mut total_points = 0usize;
    let n = runs.max(1) as usize;
    for i in 0..n {
        let prompt = prompts[i % prompts.len()];
        let t0 = Instant::now();
        let out = ArtistBrain::new().draw(prompt, i as u64 + 1);
        total_draw += t0.elapsed().as_secs_f64();
        let t1 = Instant::now();
        let pm = render(&out.document, &RenderOptions::final_image());
        total_render += t1.elapsed().as_secs_f64();
        total_strokes += out.report.strokes;
        total_points += out.report.points;
        println!(
            "run {}/{}: {:.0} ms draw, {:.0} ms render, {}x{} px, score {:.1}",
            i + 1,
            n,
            t0.elapsed().as_secs_f64() * 1000.0,
            t1.elapsed().as_secs_f64() * 1000.0,
            pm.width(),
            pm.height(),
            out.report.evaluation.score
        );
    }
    println!(
        "\navg: {:.0} ms draw · {:.0} ms render · {:.0} strokes · {:.0} points",
        total_draw / n as f64 * 1000.0,
        total_render / n as f64 * 1000.0,
        total_strokes as f64 / n as f64,
        total_points as f64 / n as f64
    );
    Ok(())
}

const PLAYER_HTML: &str = include_str!("player.html");
