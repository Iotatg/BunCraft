//! BunCraft HTTP API.
//!
//! Draw requests return the *drawing*, not just a picture: the full action
//! timeline, the report, a PNG and an SVG. The browser player replays the
//! timeline stroke by stroke.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use buncraft_brain::{ArtistBrain, DrawingReport};
use buncraft_core::document::Document;
use buncraft_render::{render, to_svg, RenderOptions};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const PLAYER: &str = include_str!("player.html");

#[derive(Default)]
struct Store {
    artworks: BTreeMap<String, Artwork>,
}

struct Artwork {
    document: Document,
    report: DrawingReport,
    png: Vec<u8>,
}

type Shared = Arc<Mutex<Store>>;

#[derive(Deserialize)]
struct DrawRequest {
    prompt: String,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    speed: Option<f64>,
    #[serde(default)]
    correct: Option<bool>,
}

#[derive(Serialize)]
struct DrawResponse {
    id: String,
    report: DrawingReport,
    timeline_url: String,
    png_url: String,
    svg_url: String,
    preview_url: String,
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .or_else(|| std::env::var("BUNCRAFT_PORT").ok()?.parse().ok())
        .unwrap_or(8080);

    let store: Shared = Arc::new(Mutex::new(Store::default()));
    let app = Router::new()
        .route("/", get(index))
        .route("/player.html", get(player))
        .route("/api/health", get(health))
        .route("/api/draw", post(draw))
        .route("/api/artworks", get(list))
        .route("/api/artworks/:id/timeline", get(timeline))
        .route("/api/artworks/:id/png", get(png))
        .route("/api/artworks/:id/svg", get(svg))
        .with_state(store);

    let addr = format!("0.0.0.0:{port}");
    println!("BunCraft API on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> &'static str {
    "ok"
}

async fn player() -> Html<&'static str> {
    Html(PLAYER)
}

async fn index() -> Html<String> {
    Html(format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>BunCraft</title>
<style>
body{{background:#14131a;color:#e8e5f0;font:14px/1.6 ui-sans-serif,system-ui,sans-serif;margin:0;padding:40px}}
h1{{font-size:20px;margin:0 0 4px}}p{{color:#9a95ac;margin:0 0 24px}}
code{{background:#201f2a;padding:2px 6px;border-radius:4px}}
a{{color:#e0604c}}
.card{{border:1px solid #2e2c39;border-radius:10px;padding:14px 16px;margin:10px 0;background:#1d1c25}}
</style></head><body>
<h1>BunCraft</h1>
<p>An AI that doesn't just generate art. It learns to draw.</p>
<div class="card">
<code>POST /api/draw</code>
<pre>{{"prompt":"silver haired girl with red eyes, shy smile","seed":7,"speed":2}}</pre>
</div>
<div id="list"></div>
<script>
fetch('/api/artworks').then(r=>r.json()).then(items=>{{
  const box=document.getElementById('list');
  if(!items.length){{box.innerHTML='<p>No drawings yet.</p>';return;}}
  box.innerHTML=items.map(a=>`<div class="card"><b>${{a.title}}</b><br>
  <span style="color:#9a95ac">${{a.prompt}}</span><br>
  ${{a.strokes}} strokes · score ${{a.score.toFixed(1)}}/100 ·
  <a href="${{a.preview_url}}">replay</a> ·
  <a href="${{a.png_url}}">png</a> ·
  <a href="${{a.svg_url}}">svg</a></div>`).join('');
}});
</script></body></html>"#
    ))
}

async fn list(State(store): State<Shared>) -> impl IntoResponse {
    let s = store.lock().await;
    let items: Vec<serde_json::Value> = s
        .artworks
        .iter()
        .map(|(id, a)| {
            serde_json::json!({
                "id": id,
                "title": a.report.title,
                "prompt": a.report.prompt,
                "strokes": a.report.strokes,
                "score": a.report.evaluation.score,
                "preview_url": format!("/player.html?src=/api/artworks/{id}/timeline"),
                "png_url": format!("/api/artworks/{id}/png"),
                "svg_url": format!("/api/artworks/{id}/svg"),
            })
        })
        .collect();
    Json(items)
}

async fn draw(State(store): State<Shared>, Json(req): Json<DrawRequest>) -> Response {
    let seed = req.seed.unwrap_or(1);
    let speed = req.speed.unwrap_or(2.0).clamp(0.1, 32.0);
    let correct = req.correct.unwrap_or(true);
    let prompt = req.prompt.clone();

    let built = tokio::task::spawn_blocking(move || {
        let mut brain = ArtistBrain::new();
        brain.speed = speed;
        brain.self_correct = correct;
        let outcome = brain.draw(&prompt, seed);
        let pm = render(&outcome.document, &RenderOptions::final_image());
        let png = pm.encode_png().ok()?;
        Some((outcome, png))
    })
    .await
    .ok()
    .flatten();

    let Some((outcome, png)) = built else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "render failed").into_response();
    };

    let id = format!(
        "{}-{}-{}",
        slug(&outcome.report.prompt),
        seed,
        store.lock().await.artworks.len() + 1
    );
    let response = DrawResponse {
        id: id.clone(),
        report: outcome.report.clone(),
        timeline_url: format!("/api/artworks/{id}/timeline"),
        png_url: format!("/api/artworks/{id}/png"),
        svg_url: format!("/api/artworks/{id}/svg"),
        preview_url: format!("/player.html?src=/api/artworks/{id}/timeline"),
    };

    store.lock().await.artworks.insert(
        id,
        Artwork {
            document: outcome.document,
            report: outcome.report,
            png,
        },
    );
    Json(response).into_response()
}

async fn timeline(State(store): State<Shared>, Path(id): Path<String>) -> Response {
    let s = store.lock().await;
    match s.artworks.get(&id) {
        Some(a) => (
            [(header::CONTENT_TYPE, "application/json")],
            a.document.to_json(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "unknown artwork").into_response(),
    }
}

async fn png(State(store): State<Shared>, Path(id): Path<String>) -> Response {
    let s = store.lock().await;
    match s.artworks.get(&id) {
        Some(a) => (
            [(header::CONTENT_TYPE, "image/png")],
            a.png.clone(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "unknown artwork").into_response(),
    }
}

async fn svg(State(store): State<Shared>, Path(id): Path<String>) -> Response {
    let s = store.lock().await;
    match s.artworks.get(&id) {
        Some(a) => (
            [(header::CONTENT_TYPE, "image/svg+xml")],
            to_svg(&a.document),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "unknown artwork").into_response(),
    }
}

fn slug(prompt: &str) -> String {
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
        "artwork".to_string()
    } else {
        s.chars().take(40).collect()
    }
}
