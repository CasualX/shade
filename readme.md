Shade
=====

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/shade.svg)](https://crates.io/crates/shade)
[![docs.rs](https://docs.rs/shade/badge.svg)](https://docs.rs/shade)

Shade is a Rust graphics library with a unified rendering API.

It currently provides OpenGL and WebGL backends, plus shared types for buffers,
textures, shaders, uniforms, images, and small rendering utilities.

Features
--------

- One graphics API across desktop OpenGL, WebGL2 and mobile
- Explicit control over buffers, textures, shaders, uniforms, and draw calls
- Helper modules for 2D, 3D, immediate-mode style rendering, images, and dithering
- Optional image loading support for PNG, GIF, and JPEG
- Optional MSDF generation support

Shade is a rendering library, not a windowing framework. You bring the GL/WebGL context and event loop, then render through Shade.

Installation
------------

```toml
[dependencies]
shade = "0.0.5"
```

Available cargo features:

- `gl` enables the OpenGL backend
- `webgl` enables the WebGL backend
- `png`, `gif`, `jpeg` enable image decoders
- `msdfgen` enables MSDF helpers
- `serde` enables serde support where available

Examples
--------

See the `examples/` directory for complete programs.

- [examples/triangle.rs](examples/triangle.rs) is the smallest desktop rendering example
- [examples/text.rs](examples/text.rs) covers text rendering work
- [examples/polygon.rs](examples/polygon.rs) draws a user editable polygon
- [examples/renderer/main.rs](examples/renderer/main.rs) contains a simple 3D renderer built on Shade
- [examples/webgl/](examples/webgl/) for WebGL examples, including live demos

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
