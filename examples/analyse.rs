//! Let the tracer look at the picture first: `analyse` names the kind of picture and proposes a
//! selection method and threshold, which this example then traces with.
//!
//!     cargo run --release --example analyse -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::{TraceParams, analyse, trace};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("analyse", "dog_picture.png")?;
    let report = analyse(picture.raster())?;
    println!(
        "type {:?}, capture {:?}, confidence {:.2}",
        report.image_type, report.capture, report.confidence
    );
    println!("reasons: {:?}", report.reasons);
    for property in &report.properties {
        println!(
            "  {:?} = {:.3} (confidence {:.2})",
            property.kind, property.value, property.confidence
        );
    }
    println!("proposal: {:?}", report.proposal);
    assert!(
        !report.properties.is_empty(),
        "the report lists its measurements"
    );

    let params = TraceParams {
        selection: report.proposal.selection,
        ..TraceParams::with_threshold(report.proposal.threshold)
    };
    let result = trace(picture.raster(), common::map(), &params, &mut |_| {
        ControlFlow::Continue(())
    })?;
    assert!(result.counts.paths > 0, "the proposal traced nothing");
    let layers = [(common::Style::Outline, result.outlines.as_slice())];
    let file = common::write_svg(&out, "proposal", &picture, &layers)?;
    println!(
        "{} paths, {} nodes -> {}",
        result.counts.paths,
        result.counts.nodes,
        file.display()
    );
    Ok(())
}
