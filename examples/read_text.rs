//! Reading text: the OCR model (weights passed in as bytes) finds lines, and handing them to
//! `Job::lines` makes the trace set them apart as text. Skips when the weights are missing.
//!
//!     cargo run --release --example read_text -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::job::{self, Job, Mode};
use vectrace::ocr::{OcrModel, read_text};
use vectrace::{TraceParams, auto_threshold, wire};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("read_text", "precision.png")?;
    let weights = common::assets().join("models/paddleocr-latin");
    common::skip_without(&weights.join("dict.txt"));
    let alphabet = std::fs::read_to_string(weights.join("dict.txt"))?;
    let model = OcrModel::load(
        &std::fs::read(weights.join("det.onnx"))?,
        &std::fs::read(weights.join("rec.onnx"))?,
        &alphabet,
    )?;

    let lines = read_text(
        &model,
        &picture.rgba,
        picture.width,
        picture.height,
        &mut |_| ControlFlow::Continue(()),
    )?;
    assert!(
        lines.iter().any(|l| l.text.contains("BERATUNG")),
        "precision.png says BERATUNG"
    );
    for line in &lines {
        println!("{:?} (accepted: {})", line.text, line.accepted());
    }

    let params = TraceParams::with_threshold(auto_threshold(picture.raster()));
    let job = Job {
        raster: picture.raster(),
        map: common::map(),
        mode: Mode::Mono(&params),
        lines: &lines,
        mask: None,
        shapes: false,
    };
    let result = wire::read(&job::run(&job, &mut |_| {})).ok_or("malformed numbers")?;
    println!(
        "{} lines found, {} set as text, {} paths",
        result.texts.found,
        result.texts.lines.len(),
        result.paths
    );
    let text: Vec<_> = result
        .texts
        .lines
        .iter()
        .flat_map(|t| t.outlines.clone())
        .collect();
    let layers = [
        (common::Style::Outline, result.outlines.as_slice()),
        (common::Style::Accent, text.as_slice()),
    ];
    common::write_svg(&out, "text", &picture, &layers)?;
    Ok(())
}
