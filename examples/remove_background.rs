//! Tracing everything that is not the background colour, for pictures with colour or tone.
//! `suggest_background` finds the colour; the colour tolerance (0 to 100) decides how far from
//! it a pixel still counts as background.
//!
//!     cargo run --release --example remove_background -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::{
    DEFAULT_BACKGROUND_TOLERANCE, Selection, TraceParams, auto_threshold, suggest_background, trace,
};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("remove_background", "elephant.jpg")?;
    let raster = picture.raster();
    let background = suggest_background(raster); // None: a transparent background
    println!("suggested background: {background:?}");
    assert!(
        background.is_some(),
        "an opaque picture has a background colour"
    );

    let base = TraceParams::with_threshold(auto_threshold(raster));
    let by = |tolerance| TraceParams {
        selection: Selection::RemoveBackground {
            background,
            tolerance,
        },
        ..base
    };
    let cases = [
        ("brightness", base),
        ("tolerance-15", by(15)),
        ("tolerance-default", by(DEFAULT_BACKGROUND_TOLERANCE)),
        ("tolerance-70", by(70)),
    ];
    for (name, params) in cases {
        let result = trace(raster, common::map(), &params, &mut |_| {
            ControlFlow::Continue(())
        })?;
        assert!(result.counts.paths > 0, "{name} traced nothing");
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        common::write_svg(&out, name, &picture, &layers)?;
        println!(
            "{name}: {} paths, {} nodes",
            result.counts.paths, result.counts.nodes
        );
    }
    Ok(())
}
