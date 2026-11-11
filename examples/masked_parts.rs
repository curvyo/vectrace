//! Keeping only part of a result: `Job::mask` takes anything that implements `Mask`. This one
//! keeps the left or the right half of the picture, so the picture is split into two parts.
//! Real masks clip outlines at the edge; this simple one keeps whole outlines by their centre.
//!
//!     cargo run --release --example masked_parts -- [--out DIR] [IMAGE]
mod common;

use kurbo::{BezPath, Point, Shape as _};
use vectrace::job::{self, Job, Mode};
use vectrace::mask::{Clipped, Mask};
use vectrace::{TraceParams, auto_threshold, wire};

/// Keeps what lies left of `x` (millimetres), or right of it.
struct Half {
    x: f64,
    left: bool,
}

impl Mask for Half {
    fn keeps(&self, p: Point) -> bool {
        (p.x <= self.x) == self.left
    }

    fn clip(&self, outlines: &[BezPath]) -> Clipped {
        let kept: Vec<BezPath> = outlines
            .iter()
            .filter(|o| self.keeps(o.bounding_box().center()))
            .cloned()
            .collect();
        Clipped {
            changed: kept.len() != outlines.len(),
            outlines: kept,
            kept_whole: 0,
        }
    }
}

fn main() -> common::Result<()> {
    let (picture, out) = common::open("masked_parts", "forms.png")?;
    let params = TraceParams::with_threshold(auto_threshold(picture.raster()));
    let middle = f64::from(picture.width) * common::MM_PER_PX / 2.0;
    for (name, left) in [("left", true), ("right", false)] {
        let mask = Half { x: middle, left };
        let job = Job {
            raster: picture.raster(),
            map: common::map(),
            mode: Mode::Mono(&params),
            lines: &[],
            mask: Some(&mask),
            shapes: false,
        };
        let result = wire::read(&job::run(&job, &mut |_| {})).ok_or("malformed numbers")?;
        let layers = [(common::Style::Outline, result.outlines.as_slice())];
        common::write_svg(&out, name, &picture, &layers)?;
        println!("{name}: {} paths, {} nodes", result.paths, result.nodes);
        assert!(result.paths > 0, "the {name} half holds shapes");
    }
    Ok(())
}
