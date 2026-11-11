//! Progress and cancelling. `job::run` reports whole percents; `trace` calls a poll closure
//! at fixed work boundaries, and `ControlFlow::Break` stops it with `TraceError::Cancelled`.
//!
//!     cargo run --release --example progress_and_cancel -- [--out DIR] [IMAGE]
mod common;

use std::ops::ControlFlow;

use vectrace::job::{self, Job, Mode};
use vectrace::{TraceError, TraceParams, auto_threshold, trace, wire};

fn main() -> common::Result<()> {
    let (picture, out) = common::open("progress_and_cancel", "testlogo.png")?;
    let params = TraceParams::with_threshold(auto_threshold(picture.raster()));

    let job = Job {
        raster: picture.raster(),
        map: common::map(),
        mode: Mode::Mono(&params),
        lines: &[],
        mask: None,
        shapes: false,
    };
    let mut last = 0;
    let numbers = job::run(&job, &mut |percent| last = percent); // the values only rise
    let result = wire::read(&numbers).ok_or("malformed numbers")?;
    assert_eq!(last, 100, "a finished job reports 100 %");
    println!("job finished at {last} %: {} paths", result.paths);
    let layers = [(common::Style::Outline, result.outlines.as_slice())];
    common::write_svg(&out, "finished", &picture, &layers)?;

    let mut polls = 0;
    let stopped = trace(picture.raster(), common::map(), &params, &mut |progress| {
        polls += 1;
        if polls == 3 {
            println!("stopping at {progress:?}");
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    assert_eq!(
        stopped.err(),
        Some(TraceError::Cancelled),
        "Break must stop the trace"
    );
    println!("trace was cancelled after {polls} polls");
    Ok(())
}
