//! Shape recognition: with `Job::shapes` the result also says which outlines are rectangles,
//! ellipses, regular polygons or stars. This goes through `job::run`, the flat-number entry
//! point, and `wire::read` to get the result back.
//!
//!     cargo run --release --example recognise_shapes -- [--out DIR] [IMAGE]
mod common;

use vectrace::job::{self, Job, Mode};
use vectrace::{TraceParams, auto_threshold, wire};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("recognise_shapes", "forms.png")?;
    let params = TraceParams::with_threshold(auto_threshold(picture.raster()));
    let job = Job {
        raster: picture.raster(),
        map: common::map(),
        mode: Mode::Mono(&params),
        lines: &[],
        mask: None,
        shapes: true,
    };
    let numbers = job::run(&job, &mut |_| {});
    let result = wire::read(&numbers).ok_or("the worker numbers are malformed")?;
    println!("{} paths, {} nodes", result.paths, result.nodes);
    assert!(
        result.shapes.iter().any(|r| r.primitives() > 0),
        "forms.png contains primitives"
    );

    let mut recognised = Vec::new();
    for region in &result.shapes {
        for (primitive, _rotation) in region.shapes.iter().flatten() {
            println!("{primitive:?}");
        }
        if region.primitives() > 0 {
            recognised.extend(
                region
                    .outlines
                    .iter()
                    .filter_map(|&i| result.outlines.get(i))
                    .cloned(),
            );
        }
    }
    let layers = [
        (common::Style::Outline, result.outlines.as_slice()),
        (common::Style::Accent, recognised.as_slice()),
    ];
    let file = common::write_svg(&out, "shapes", &picture, &layers)?;
    println!(
        "{} regions are primitives -> {}",
        result.shapes.iter().filter(|r| r.primitives() > 0).count(),
        file.display()
    );
    Ok(())
}
