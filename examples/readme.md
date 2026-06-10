Examples
========

This directory holds the sample projects for `shade`.

Most of the small demos live in [`demos/`](demos/). The other example projects are shells or wrappers around those demos, plus one larger standalone renderer.

Projects
--------

### `demos/`

Shared demo code used by the desktop and web examples.

Notable files:

- [`demos/src/examples/triangle.rs`](demos/src/examples/triangle.rs) is the smallest rendering example
- [`demos/src/examples/text.rs`](demos/src/examples/text.rs) covers text rendering with MSDF assets
- [`demos/src/examples/polygon.rs`](demos/src/examples/polygon.rs) is a more interactive 2D example
- [`demos/src/examples/globe.rs`](demos/src/examples/globe.rs) 3D globe with camera controls

### `desktop/`

Native desktop shell for the shared demos using `winit` + `glutin` and Shade's OpenGL backend.

It loads assets from the repo at runtime, opens a window, forwards input events, and can launch individual demos by name.

Run it with:

```bash
cargo run -p desktop
cargo run -p desktop -- triangle
cargo run -p desktop -- text
cargo run -p desktop -- scene
```

Running `cargo run -p desktop` prints the available demo ids. Pass one of them after `--` to launch it.

### `webgl/`

Browser shell for the same demo set, built as a wasm module on top of Shade's WebGL backend.

The Rust side exposes one constructor per demo, while the HTML/JS shell handles canvas resize, input, redraw scheduling and browser file picking. It is effectively the web counterpart to `desktop/`.

Build the wasm bundle with `./examples/webgl/build.sh` (linux) or `examples\webgl\build.bat` (windows), then open or serve `examples/webgl/html/index.html` in a browser.

There is also a hosted demo page linked from [`webgl/readme.md`](webgl/readme.md).

### `renderer/`

Standalone integrated 3D example that is separate from the shared demo system.

[`renderer/main.rs`](renderer/main.rs) sets up a more involved scene with an arcball camera, shadow mapping, frustum culling, and multiple renderables.

This is the example to read if you want to see a bigger "real project" style use of Shade rather than a single focused demo.
