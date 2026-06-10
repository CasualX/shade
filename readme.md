Shade
=====

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/shade.svg)](https://crates.io/crates/shade)
[![docs.rs](https://docs.rs/shade/badge.svg)](https://docs.rs/shade)

Shade is a small Rust graphics library with a unified rendering API.

It currently targets desktop OpenGL and WebGL, with shared types for buffers,
textures, shaders, uniforms, images, and a handful of higher level rendering helpers.

What It Includes
----------------

- One API across desktop OpenGL and WebGL
- Low level rendering building blocks: buffers, textures, shaders, uniforms and draw calls
- Utility modules for 2D, 3D, immediate mode drawing, images and dithering
- Optional image decoding for PNG, GIF and JPEG
- Optional MSDF helpers for text workflows

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

Project Layout
--------------

- [`src/`](src/) contains the library itself
- [`examples/`](examples/) contains runnable sample projects and demo code
- [`docs.rs/shade`](https://docs.rs/shade) has the API docs

If you want to see Shade in use, start with [`examples/readme.md`](examples/readme.md). The smallest code sample is [`examples/demos/src/examples/triangle.rs`](examples/demos/src/examples/triangle.rs).

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
