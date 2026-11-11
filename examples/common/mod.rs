//! The helpers every example shares: decode a picture into a [`vectrace::Raster`]'s bytes, write
//! an SVG with the picture washed out and the outlines on top, and skip when assets are missing.

#![allow(dead_code)] // each example uses some of these

use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageDecoder as _, ImageReader, metadata::Orientation};
use kurbo::BezPath;
use vectrace::{PixelMap, Raster, Rgb8};

/// Placing a picture at 96 dpi: one pixel is 25.4 / 96 mm.
pub const MM_PER_PX: f64 = 25.4 / 96.0;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// A decoded picture (RGBA8, straight alpha, upright) and where it came from.
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub source: PathBuf,
}

impl Picture {
    pub fn raster(&self) -> Raster<'_> {
        Raster {
            width: self.width,
            height: self.height,
            rgba: &self.rgba,
        }
    }
}

/// Pixels to millimetres at 96 dpi.
pub fn map() -> PixelMap {
    PixelMap::scale(MM_PER_PX, MM_PER_PX)
}

/// How a trace is drawn on the picture.
pub enum Style {
    /// A thin blue outline on a white casing, a faint fill.
    Outline,
    /// A highlighted outline.
    Accent,
    /// Filled with a colour.
    Fill(Rgb8),
}

/// Reads `[--out DIR] [IMAGE]` from the command line: the picture (`default_image` from the
/// reference images when none is given) and the directory for the SVGs
/// (`target/examples/<example>` when `--out` is not given).
pub fn open(example: &str, default_image: &str) -> Result<(Picture, PathBuf)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = root.join("target/examples").join(example);
    let mut image = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--list" {
            // `cargo nextest` asks a test binary for its tests; an example has none to list.
            std::process::exit(0);
        } else if arg == "--out" {
            out = PathBuf::from(args.next().ok_or("--out needs a directory")?);
        } else if !arg.starts_with('-') {
            image = Some(PathBuf::from(arg));
        }
    }
    let image = image.unwrap_or_else(|| assets().join("trace-references").join(default_image));
    skip_without(&image);
    std::fs::create_dir_all(&out)?;
    Ok((decode(&image)?, out))
}

/// A missing asset skips the example (exit 0 with a message), or fails it when
/// `VECTRACE_REQUIRE_ASSETS` is set, as the tests do.
pub fn skip_without(path: &Path) {
    if path.exists() {
        return;
    }
    println!(
        "skipped: {} not found (set VECTRACE_ASSETS to the asset directory)",
        path.display()
    );
    std::process::exit(i32::from(
        std::env::var_os("VECTRACE_REQUIRE_ASSETS").is_some(),
    ));
}

/// The asset directory: `$VECTRACE_ASSETS` or `assets/` of this repository.
pub fn assets() -> PathBuf {
    std::env::var_os("VECTRACE_ASSETS").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
        PathBuf::from,
    )
}

/// Decodes a PNG or JPEG, applying its EXIF orientation.
pub fn decode(path: &Path) -> Result<Picture> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    let rgba = image.into_rgba8();
    Ok(Picture {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
        source: path.to_owned(),
    })
}

/// Writes `dir/name.svg`: the picture washed out (a copy next to it, `source.<ext>`), the
/// `layers` of outlines in millimetres on top. Returns the file.
pub fn write_svg(
    dir: &Path,
    name: &str,
    picture: &Picture,
    layers: &[(Style, &[BezPath])],
) -> Result<PathBuf> {
    let ext = picture
        .source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let copy = format!("source.{ext}");
    std::fs::copy(&picture.source, dir.join(&copy))?;
    let (w, h) = (
        f64::from(picture.width) * MM_PER_PX,
        f64::from(picture.height) * MM_PER_PX,
    );
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w:.3} {h:.3}\" width=\"{w:.3}mm\" height=\"{h:.3}mm\">\n"
    );
    let _ = writeln!(
        svg,
        "<rect width=\"{w:.3}\" height=\"{h:.3}\" fill=\"#fff\"/>"
    );
    let _ = writeln!(
        svg,
        "<image href=\"{copy}\" width=\"{w:.3}\" height=\"{h:.3}\" opacity=\".3\"/>"
    );
    let stroke =
        "fill-rule=\"evenodd\" vector-effect=\"non-scaling-stroke\" stroke-linejoin=\"round\"";
    for (i, (style, outlines)) in layers.iter().enumerate() {
        let d = outlines
            .iter()
            .map(BezPath::to_svg)
            .collect::<Vec<_>>()
            .join(" ");
        let _ = writeln!(svg, "<defs><path id=\"p{i}\" {stroke} d=\"{d}\"/></defs>");
        match style {
            Style::Outline => {
                let _ = writeln!(
                    svg,
                    "<use href=\"#p{i}\" fill=\"#2f6fdd\" fill-opacity=\".16\"/>"
                );
                let _ = writeln!(
                    svg,
                    "<use href=\"#p{i}\" fill=\"none\" stroke=\"#fff\" stroke-width=\"3\"/>"
                );
                let _ = writeln!(
                    svg,
                    "<use href=\"#p{i}\" fill=\"none\" stroke=\"#1f5fd0\" stroke-width=\"1\"/>"
                );
            }
            Style::Accent => {
                let _ = writeln!(
                    svg,
                    "<use href=\"#p{i}\" fill=\"#f76707\" fill-opacity=\".3\" stroke=\"#e8590c\" stroke-width=\"2\"/>"
                );
            }
            Style::Fill(c) => {
                let _ = writeln!(
                    svg,
                    "<use href=\"#p{i}\" fill=\"#{0:02x}{1:02x}{2:02x}\" stroke=\"#{0:02x}{1:02x}{2:02x}\"/>",
                    c.r, c.g, c.b
                );
            }
        }
    }
    svg.push_str("</svg>\n");
    let file = dir.join(format!("{name}.svg"));
    std::fs::write(&file, svg)?;
    Ok(file)
}
