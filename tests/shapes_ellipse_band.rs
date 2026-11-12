//! A stroked ellipse (a thin ring between two ellipses) is recognised as an ellipse primitive with
//! the stroke's width, on a rendered ring and on the customer's `forms2.png`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::too_many_arguments
)]

#[path = "common/assets.rs"]
mod assets;

use vectrace::job::{self, Job, Mode};
use vectrace::shapes::{Band, Primitive};
use vectrace::{PixelMap, Raster, TraceParams, auto_threshold, wire};

const PIXEL_MM: f64 = 25.4 / 96.0;

/// The bands of a picture, as the worker returns them.
fn bands(raster: Raster<'_>) -> Vec<Band> {
    let params = TraceParams::with_threshold(auto_threshold(raster));
    let job = Job {
        raster,
        map: PixelMap::scale(PIXEL_MM, PIXEL_MM),
        mode: Mode::Mono(&params),
        lines: &[],
        mask: None,
        shapes: true,
    };
    let numbers = job::run(&job, &mut |_| {});
    let result = wire::read(&numbers).expect("malformed numbers");
    result.shapes.iter().filter_map(|r| r.band).collect()
}

/// A black ring of `width` px about the ellipse (`rx`, `ry`) on a white `size` square, 4x4
/// supersampled.
fn ring(size: u32, (cx, cy): (f64, f64), (rx, ry): (f64, f64), width: f64) -> Vec<u8> {
    let mut rgba = Vec::new();
    for y in 0..size {
        for x in 0..size {
            let mut hit = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) / 4.0 - cx;
                    let py = f64::from(y) + (f64::from(sy) + 0.5) / 4.0 - cy;
                    // The ring between the offsets of the ellipse, measured along the normal
                    // by the gradient of the implicit function.
                    let f = (px / rx).powi(2) + (py / ry).powi(2);
                    let grad = 2.0 * ((px / (rx * rx)).powi(2) + (py / (ry * ry)).powi(2)).sqrt();
                    if (f - 1.0).abs() / grad <= width / 2.0 {
                        hit += 1;
                    }
                }
            }
            let v = 255 - (255 * hit / 16) as u8;
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
    }
    rgba
}

fn check(found: &[Band], (rx, ry): (f64, f64), width: f64) {
    assert_eq!(found.len(), 1, "{found:?}");
    let Primitive::Ellipse { rx: a, ry: b, .. } = found[0].primitive else {
        panic!("{:?}", found[0]);
    };
    // The fit may call the long axis rx and turn the ellipse a quarter: compare the sorted radii.
    let (a, b) = (a.as_mm() / PIXEL_MM, b.as_mm() / PIXEL_MM);
    let (long, short) = (a.max(b), a.min(b));
    let w = found[0].width.as_mm() / PIXEL_MM;
    assert!(
        (long - rx.max(ry)).abs() < 0.5
            && (short - rx.min(ry)).abs() < 0.5
            && (w - width).abs() < 0.5,
        "radii {a} {b} width {w}"
    );
}

fn render(size: u32, centre: (f64, f64), radii: (f64, f64), width: f64) -> Vec<Band> {
    let rgba = ring(size, centre, radii, width);
    bands(Raster {
        width: size,
        height: size,
        rgba: &rgba,
    })
}

#[test]
fn a_thin_ellipse_ring_is_a_stroked_ellipse() {
    check(
        &render(300, (100.0, 150.0), (22.0, 110.0), 3.6),
        (22.0, 110.0),
        3.6,
    );
}

#[test]
fn a_circle_ring_is_a_stroked_circle() {
    check(
        &render(300, (150.0, 150.0), (80.0, 80.0), 6.0),
        (80.0, 80.0),
        6.0,
    );
}

#[test]
fn a_wide_ellipse_ring_is_a_stroked_ellipse() {
    check(
        &render(300, (150.0, 150.0), (120.0, 70.0), 14.0),
        (120.0, 70.0),
        14.0,
    );
}

#[test]
fn forms2_has_the_thin_vertical_ellipse_as_a_stroked_ellipse() {
    let Some(dir) = assets::references() else {
        return;
    };
    let picture = image::open(dir.join("forms2.png")).unwrap().to_rgba8();
    let found = bands(Raster {
        width: picture.width(),
        height: picture.height(),
        rgba: picture.as_raw(),
    });
    let thin: Vec<&Band> = found
        .iter()
        .filter(|b| match b.primitive {
            Primitive::Ellipse { rx, ry, .. } => {
                rx.as_mm().max(ry.as_mm()) > 2.0 * rx.as_mm().min(ry.as_mm())
            }
            _ => false,
        })
        .collect();
    assert_eq!(thin.len(), 1, "{found:?}");
    // The ring of the left, thin ellipse: centre (26.6, 141.4) mm, radii 20.8 and 6.7 mm, stroke
    // 0.97 mm (3.7 px; the radii's difference said 4.15 px and failed the area check).
    let Primitive::Ellipse { centre, rx, ry } = thin[0].primitive else {
        unreachable!();
    };
    let (long, short) = (rx.as_mm().max(ry.as_mm()), rx.as_mm().min(ry.as_mm()));
    assert!((centre.x - 26.6).abs() < 0.1 && (centre.y - 141.4).abs() < 0.1);
    assert!((long - 20.8).abs() < 0.1 && (short - 6.7).abs() < 0.1);
    assert!((thin[0].width.as_mm() - 0.97).abs() < 0.05);
}
