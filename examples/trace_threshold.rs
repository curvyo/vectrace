//! Black-and-white tracing by brightness: the automatic threshold, a manual one, and invert.
//!
//!     cargo run --release --example trace_threshold -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;
use std::time::Instant;

use vectrace::{TraceParams, auto_threshold, trace};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("trace_threshold", "testlogo.png")?;
    let raster = picture.raster();
    let auto = auto_threshold(raster); // Otsu's cut, 0 to 255
    println!("automatic threshold: {auto}");

    let cases = [
        ("auto", TraceParams::with_threshold(auto)),
        ("manual-80", TraceParams::with_threshold(80)),
        (
            "auto-inverted",
            TraceParams {
                invert: true,
                ..TraceParams::with_threshold(auto)
            },
        ),
    ];
    let mut foreground = Vec::new();
    for (name, params) in cases {
        let started = Instant::now();
        // `poll` is called while the trace runs; Continue keeps going.
        let result = trace(raster, common::map(), &params, &mut |_| {
            ControlFlow::Continue(())
        })?;
        foreground.push(result.foreground_pixels);
        assert!(result.counts.paths > 0, "{name} traced nothing");
        let ms = started.elapsed().as_millis();
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        let file = common::write_svg(&out, name, &picture, &layers)?;
        println!(
            "{name}: {} paths, {} nodes, {ms} ms -> {}",
            result.counts.paths,
            result.counts.nodes,
            file.display()
        );
    }
    assert_ne!(
        foreground[0], foreground[2],
        "invert must change what is content"
    );
    Ok(())
}
