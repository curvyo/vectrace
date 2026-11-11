//! One layer per colour: the automatic colour count, a fixed count, a colour you fix, and a
//! palette colour excluded. Here every opaque pixel is content (the background is "transparent"),
//! so the paper colour is in the palette and can be excluded.
//!
//!     cargo run --release --example multicolour -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::{ColourParams, Selection, TraceParams, sample_colour, trace_colours};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("multicolour", "blütenzeit.png")?;
    let raster = picture.raster();
    let content = TraceParams {
        selection: Selection::RemoveBackground {
            background: None,
            tolerance: 40,
        },
        ..TraceParams::with_threshold(128)
    };
    let run = |name: &str, params: &ColourParams| -> common::Result<vectrace::ColourTrace> {
        let result = trace_colours(raster, common::map(), params, &mut |_| {
            ControlFlow::Continue(())
        })?;
        let layers: Vec<_> = result
            .layers
            .iter()
            .map(|l| (common::Style::Fill(l.sample), l.outlines.as_slice()))
            .collect();
        common::write_svg(&out, name, &picture, &layers)?;
        println!(
            "{name}: {} layers, {} paths, {} nodes",
            result.layers.len(),
            result.counts.paths,
            result.counts.nodes
        );
        assert!(!result.layers.is_empty(), "{name} made no layers");
        assert!(!result.layers.is_empty(), "{name} made no layers");
        Ok(result)
    };

    let auto = run("auto", &ColourParams::new(content))?;
    run(
        "colours-4",
        &ColourParams {
            colours: Some(4),
            ..ColourParams::new(content)
        },
    )?;
    run(
        "colours-8",
        &ColourParams {
            colours: Some(8),
            ..ColourParams::new(content)
        },
    )?;
    if let Some(pick) = sample_colour(raster, picture.width / 2, picture.height / 2) {
        run(
            "fixed-centre",
            &ColourParams {
                colours: Some(4),
                fixed: vec![pick],
                ..ColourParams::new(content)
            },
        )?;
    }
    // The palette is sorted fixed colours first, then by pixels taken: exclude the largest.
    if let Some(largest) = auto.palette.iter().max_by_key(|e| e.pixels) {
        println!("excluding {:?} ({} px)", largest.colour, largest.pixels);
        let excluded = run(
            "excluded",
            &ColourParams {
                excluded: vec![largest.colour],
                ..ColourParams::new(content)
            },
        )?;
        assert!(
            excluded.layers.len() < auto.layers.len(),
            "excluding a colour drops its layer"
        );
    }
    Ok(())
}
