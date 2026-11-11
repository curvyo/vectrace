//! The four presets (Logo, Photo, Silhouette, Engraving) on the same picture and threshold. A
//! preset sets smooth, tolerance, ignore below, corners, upscale and denoise, never the threshold.
//!
//!     cargo run --release --example presets -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;
use std::time::Instant;

use vectrace::{PRESETS, TraceParams, auto_threshold, trace};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("presets", "testlogo.png")?;
    let raster = picture.raster();
    let base = TraceParams::with_threshold(auto_threshold(raster));
    let mut nodes = Vec::new();
    for preset in PRESETS {
        let started = Instant::now();
        let result = trace(raster, common::map(), &preset.apply(base), &mut |_| {
            ControlFlow::Continue(())
        })?;
        nodes.push(result.counts.nodes);
        assert!(result.counts.paths > 0, "{} traced nothing", preset.key);
        let ms = started.elapsed().as_millis();
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        common::write_svg(&out, preset.key, &picture, &layers)?;
        println!(
            "{:<10} {} paths, {} nodes, {} specks removed, {ms} ms",
            preset.key, result.counts.paths, result.counts.nodes, result.counts.specks_removed
        );
    }
    assert!(
        nodes[2] < nodes[3],
        "Silhouette should use fewer nodes than Engraving"
    );
    Ok(())
}
