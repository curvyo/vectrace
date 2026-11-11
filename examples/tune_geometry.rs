//! The geometry settings one at a time, three values each, from the Logo preset: smooth (0 to
//! 100), tolerance (source px), ignore below (specks, mm), corners (0 to 100), denoise, upscale.
//!
//!     cargo run --release --example tune_geometry -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::{Denoise, Mm, PRESETS, PixelLength, TraceParams, Upscale, auto_threshold, trace};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("tune_geometry", "dog_lines.png")?;
    let raster = picture.raster();
    let logo = PRESETS[0].apply(TraceParams::with_threshold(auto_threshold(raster)));

    let mut cases: Vec<(String, TraceParams)> = Vec::new();
    for smooth in [0, 50, 100] {
        cases.push((format!("smooth-{smooth}"), TraceParams { smooth, ..logo }));
    }
    for px in [0.35, 0.5, 1.5] {
        cases.push((
            format!("tolerance-{px}"),
            TraceParams {
                tolerance: PixelLength::from_px(px),
                ..logo
            },
        ));
    }
    for mm in [0.1, 0.5, 2.0] {
        cases.push((
            format!("specks-{mm}"),
            TraceParams {
                specks: Mm::from_mm(mm),
                ..logo
            },
        ));
    }
    for corners in [10, 40, 90] {
        cases.push((
            format!("corners-{corners}"),
            TraceParams { corners, ..logo },
        ));
    }
    for denoise in [Denoise::Off, Denoise::Low, Denoise::High] {
        cases.push((
            format!("denoise-{denoise:?}"),
            TraceParams { denoise, ..logo },
        ));
    }
    // Upscale has two settings; Auto traces a 2x copy of pictures under one megapixel.
    for upscale in [Upscale::Off, Upscale::Auto] {
        cases.push((
            format!("upscale-{upscale:?}"),
            TraceParams { upscale, ..logo },
        ));
    }
    let (mut nodes, mut paths) = (
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    for (name, params) in cases {
        let result = trace(raster, common::map(), &params, &mut |_| {
            ControlFlow::Continue(())
        })?;
        nodes.insert(name.clone(), result.counts.nodes);
        paths.insert(name.clone(), result.counts.paths);
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        common::write_svg(&out, &name, &picture, &layers)?;
        println!(
            "{name:<16} {} paths, {} nodes, {} specks removed",
            result.counts.paths, result.counts.nodes, result.counts.specks_removed
        );
    }
    assert!(
        nodes["smooth-0"] > nodes["smooth-100"],
        "more smoothing, fewer nodes"
    );
    assert!(
        nodes["tolerance-0.35"] > nodes["tolerance-1.5"],
        "larger tolerance, fewer nodes"
    );
    assert!(
        paths["specks-0.1"] > paths["specks-2"],
        "a larger ignore-below removes more shapes"
    );
    Ok(())
}
