//! The entry point for a complete trace job: [`run`] takes a [`Job`] (picture, settings, optional
//! OCR lines, optional mask, shape recognition on or off) and returns the result as flat numbers.
//!
//! Read the numbers back with [`crate::wire::read`]; the layout is described in [`crate::wire`].
//! [`text_regions`] runs only the text pass. For a plain trace without OCR, masks or shapes,
//! [`crate::trace`] and [`crate::trace_colours`] return typed results directly.

mod text;
mod text_colour;

use crate::{ColourParams, PixelMap, Raster, TraceParams};

use self::text::{text_mono_wire, text_regions_wire};
use self::text_colour::text_colour_wire;
use crate::mask::Mask;
use crate::ocr::OcrLine;
use crate::shapes::add_shapes;

/// The settings of a job: one black result or one per colour.
#[derive(Debug, Clone, Copy)]
pub enum Mode<'a> {
    /// One result in black.
    Mono(&'a TraceParams),
    /// One layer per palette colour.
    Colours(&'a ColourParams),
}

/// One trace job: what [`run`] needs.
#[derive(Clone, Copy)]
pub struct Job<'a> {
    /// The picture, straight alpha.
    pub raster: Raster<'a>,
    /// Maps the picture's pixels to millimetres.
    pub map: PixelMap,
    /// The settings.
    pub mode: Mode<'a>,
    /// The OCR lines in the picture's pixels; none for no text pass.
    pub lines: &'a [OcrLine],
    /// The area the result keeps; `None` keeps all of it.
    pub mask: Option<&'a dyn Mask>,
    /// Recognise shapes (rectangles, ellipses, polygons, stars) and add their section to the result.
    pub shapes: bool,
}

/// The result of `job` as numbers, never panicking on bad input (it says `Invalid`).
/// `on_percent` is called each time the job reaches a higher whole percent; the
/// values only rise. With lines the text pass runs first; with no lines the numbers are exactly
/// those of the plain trace.
#[must_use]
pub fn run(job: &Job<'_>, on_percent: &mut dyn FnMut(u8)) -> Vec<f64> {
    let mut wire = match job.mode {
        Mode::Mono(params) => {
            text_mono_wire(job.raster, job.map, params, job.lines, job.mask, on_percent)
        }
        Mode::Colours(params) => {
            text_colour_wire(job.raster, job.map, params, job.lines, job.mask, on_percent)
        }
    };
    if job.shapes {
        add_shapes(&mut wire, job.map);
    }
    wire
}

/// The text pass of `job` alone: the numbers of a result without outlines, then the text section of the accepted
/// lines the mask keeps, as [`run`] would write it.
#[must_use]
pub fn text_regions(job: &Job<'_>) -> Vec<f64> {
    let params = match job.mode {
        Mode::Mono(params) => params,
        Mode::Colours(params) => &params.trace,
    };
    text_regions_wire(job.raster, job.map, params, job.lines, job.mask)
}
