//! Tracing only the colours you pick: `sample_colour` reads a colour at a pixel (as a pipette
//! click would), the picked colours become swatches, and pixels near them are the content.
//!
//!     cargo run --release --example pick_colours -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::{
    DEFAULT_PICK_TOLERANCE, Rgb8, Selection, Swatches, TraceParams, sample_colour,
    suggest_background, trace,
};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("pick_colours", "elephant.jpg")?;
    let raster = picture.raster();
    let background = suggest_background(raster);
    // A pipette click on the yellow badge (13/100, 95/100 of the picture), and a dark ink colour.
    let yellow = sample_colour(raster, picture.width * 13 / 100, picture.height * 95 / 100);
    let picked: Vec<_> = yellow.into_iter().chain([Rgb8::new(40, 40, 40)]).collect();
    assert!(yellow.is_some(), "the click must hit an opaque pixel");
    println!("picked: {picked:?}");

    let mut cases = vec![("all".to_owned(), picked.clone())];
    cases.extend(
        picked
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("only-{i}"), vec![*c])),
    );
    for (name, colours) in cases {
        let params = TraceParams {
            selection: Selection::PickColours {
                swatches: Swatches::from_slice(&colours),
                background,
                tolerance: DEFAULT_PICK_TOLERANCE,
            },
            ..TraceParams::with_threshold(128)
        };
        let result = trace(raster, common::map(), &params, &mut |_| {
            ControlFlow::Continue(())
        })?;
        assert!(result.counts.paths > 0, "{name} traced nothing");
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        common::write_svg(&out, &name, &picture, &layers)?;
        println!(
            "{name}: {} paths, {} nodes",
            result.counts.paths, result.counts.nodes
        );
    }
    Ok(())
}
