//! A raster tracer and text reader: an RGBA8 picture and trace settings in, closed outlines in
//! millimetres out. Pure Rust, no filesystem, network, clock or threads, and the same result on
//! every platform including `wasm32-unknown-unknown`.
//!
//! A trace turns the pixels a *selection method* calls content (darker than a threshold, unlike
//! the background colour, or near picked colours) into outlines: region labelling, specks
//! removal, one crack-edge chain per boundary, structural corners found before any smoothing,
//! smoothing with the corners pinned, lines and cubic Béziers fitted within a tolerance, and a
//! guard against crossing outlines. A multicolour trace does this per palette colour and lets
//! neighbours share their boundary.
//!
//! # Which function for which use case
//!
//! | You want | Use | Example |
//! |---|---|---|
//! | Black-and-white outlines by brightness | [`trace`] with [`TraceParams::with_threshold`] and [`auto_threshold`] | `trace_threshold` |
//! | Everything that is not the background | [`suggest_background`], [`Selection::RemoveBackground`] | `remove_background` |
//! | Only some colours | [`sample_colour`], [`Selection::PickColours`] | `pick_colours` |
//! | Settings for cutting, carving, engraving, photos | [`PRESETS`] | `presets` |
//! | Smoother, simpler or finer outlines | the fields of [`TraceParams`] | `tune_geometry` |
//! | One layer per colour | [`trace_colours`] with [`ColourParams`] | `multicolour` |
//! | Circles, rectangles, polygons and stars as primitives | [`job::run`] with `Job::shapes`, [`wire::read`] | `recognise_shapes` |
//! | A suggested method and threshold for an unknown picture | [`analyse`] | `analyse` |
//! | Keep only part of the result | [`mask::Mask`] in [`job::Job`] | `masked_parts` |
//! | Text as text (OCR, weights passed in as bytes) | [`ocr::OcrModel`], [`ocr::read_text`], `Job::lines` | `read_text` |
//! | A progress bar and a cancel button | the `poll` closure of [`trace`], `on_percent` of [`job::run`] | `progress_and_cancel` |
//!
//! The examples are in `examples/`; run one with `cargo run --release --example trace_threshold`.
//! They write SVGs to `target/examples/<name>/`.
//!
//! # Quickstart
//!
//! ```
//! use std::ops::ControlFlow;
//! use vectrace::{PixelMap, Raster, TraceParams, trace};
//!
//! // A 20 x 20 px picture: a black square on white.
//! let mut rgba = vec![255_u8; 20 * 20 * 4];
//! for y in 5..15 {
//!     for x in 5..15 {
//!         rgba[(y * 20 + x) * 4..][..3].fill(0);
//!     }
//! }
//! let raster = Raster { width: 20, height: 20, rgba: &rgba };
//! // Place the picture at 96 dpi: one pixel is 25.4 / 96 mm.
//! let map = PixelMap::scale(25.4 / 96.0, 25.4 / 96.0);
//! // Pixels darker than 128 are content.
//! let params = TraceParams::with_threshold(128);
//! let result = trace(raster, map, &params, &mut |_| ControlFlow::Continue(())).unwrap();
//! assert_eq!(result.counts.paths, 1);
//! ```
//!
//! # Conventions
//!
//! Outlines are [`kurbo::BezPath`]s in millimetres with y pointing down, like the picture's rows;
//! holes are wound against their outer outline, so the even-odd and non-zero fill rules agree.
//! Sizes in settings are named by their unit: `Mm` for millimetres, [`PixelLength`] for pixels of
//! the picture before Upscale. A function given bytes that do not match the size, or a singular
//! map, returns [`TraceError::Invalid`] (or `status` 2 in [`wire::Output`]) and never panics.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod analysis;
mod background;
mod boundary;
mod clearance;
mod colour_map;
mod colour_trace;
mod contour;
mod corner_place;
mod corners;
mod edges;
mod filter;
mod fit;
mod guard;
mod guarded;
mod inpaint;
pub mod job;
mod line;
pub mod mask;
mod method;
pub mod ocr;
mod outline;
mod palette;
mod params;
mod picker;
mod progress;
mod raster;
mod regions;
mod schema;
mod segment;
mod select;
pub mod shapes;
mod shared_fit;
mod smooth;
mod text_pass;
mod trace;
mod units;
pub mod wire;

pub use analysis::{
    AnalysisReport, Capture, ImageType, Property, PropertyKind, Proposal, Reason, analyse,
};
pub use background::{sample_colour, suggest_background};
pub use colour_trace::{
    ColourLayer, ColourParams, ColourTrace, PaletteEntry, colour_palette, trace_colours,
};
pub use outline::node_count;
pub use palette::MAX_COLOURS;
pub use params::{
    Denoise, MAX_SWATCHES, PRESETS, PixelLength, Preset, Rgb8, Selection, Swatches, TraceParams,
    Upscale,
};
pub use progress::{Phase, Progress};
pub(crate) use raster::luma;
pub use raster::{Raster, auto_threshold};
pub use schema::{
    ADVANCED_PARAMS, DEFAULT_SMOOTH, DEFAULT_SPECKS_MM, DEFAULT_THRESHOLD, DEFAULT_TOLERANCE_PX,
    METHOD_PARAMS, PARAMS, ParamKind, ParamSpec, ParamUnit,
};
pub use select::{DEFAULT_BACKGROUND_TOLERANCE, DEFAULT_PICK_TOLERANCE, selected};
pub use trace::{Counts, Diagnostics, MAX_HOLES_PER_REGION, TraceError, TraceResult, trace};
pub use units::{Angle, Mm, PixelMap};
pub use wire::Output;
