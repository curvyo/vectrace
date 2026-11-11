# vectrace

A raster tracer and text reader in pure Rust. An RGBA8 raster and trace settings go in, closed
outlines in millimetres come out. Colour tracing, an analysis that proposes settings and an
OCR text reader (PaddleOCR through `rten`) are included. No filesystem, network, clock or
threads in the tracer; it builds for `wasm32-unknown-unknown` and gives the same result there.

## Install

```toml
[dependencies]
vectrace = { git = "https://github.com/curvyo/vectrace", tag = "v0.1.0" }
```

## Quickstart

```rust
let raster = Raster { width, height, rgba: &rgba };                     // RGBA8, straight alpha
let map = PixelMap::scale(25.4 / 96.0, 25.4 / 96.0);                    // pixels to millimetres
let params = TraceParams::with_threshold(auto_threshold(raster));       // or a preset from PRESETS
let result = trace(raster, map, &params, &mut |_| ControlFlow::Continue(()))?;
for outline in &result.outlines { /* closed kurbo::BezPath in millimetres */ }
```

`trace` and `trace_colours` return typed results. `job::run` returns the same result (with
optional OCR lines, a mask and shape recognition) as flat numbers for crossing a thread or
WebAssembly boundary; `wire::read` decodes them. The closure is called while a trace runs;
return `ControlFlow::Break(())` to stop it. See the crate documentation (`cargo doc --open`).

## Examples

Each example is a small program for one use case. It takes an optional image path (a file of
`assets/trace-references/` by default) and `--out DIR`, writes SVGs to `target/examples/<name>/`
(the picture washed out, the outlines on top) and prints paths, nodes and times.

```text
cargo run --release --example trace_threshold -- [--out DIR] [IMAGE]
```

| Use case | Example |
|---|---|
| Brightness: automatic and manual threshold, invert | `trace_threshold` |
| Everything that is not the background colour | `remove_background` |
| Only the colours you pick | `pick_colours` |
| The four presets side by side | `presets` |
| Smooth, tolerance, ignore below, corners, denoise, upscale | `tune_geometry` |
| One layer per colour; fix or exclude colours | `multicolour` |
| Rectangles, ellipses, polygons and stars as primitives | `recognise_shapes` |
| A suggested method and threshold for a picture | `analyse` |
| Keep only part of the result (your own `Mask`) | `masked_parts` |
| Read text with the OCR model and set it apart | `read_text` |
| Progress and cancel | `progress_and_cancel` |

The examples run under `cargo test` (and `cargo test --examples`) and check true facts about
their output with `assert!`. `cargo nextest` does not run example binaries, so CI runs both.

## Performance and WebAssembly

The tracer is pure: no filesystem, network, clock or threads, and it builds for
`wasm32-unknown-unknown` with the same result as native. A 1920 x 1080 picture traces in about
70 ms in a release build. Build with `--release` (or optimise the crate in the dev profile, as
`Cargo.toml` does here); an unoptimised build is many times slower. The OCR model runs on the
calling thread in WebAssembly.

## Licence

MIT OR Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.

## Test assets

The OCR weights and the reference images used by the tests are in `assets/`; their licences
and sources are in [ASSETS.md](ASSETS.md). `VECTRACE_ASSETS` points the tests at another
directory with the same layout; a missing file skips the test with a message unless
`VECTRACE_REQUIRE_ASSETS` is set, then it fails.
