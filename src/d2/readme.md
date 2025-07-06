Immediate mode 2D rendering.

The `d2` module provides a simple and ergonomic API for drawing 2D shapes, lines, text, and sprites.
It follows an immediate-mode model: geometry is generated and drawn every frame,
with optional batching optimizations behind the scenes.

## Getting started

To generate draw commands of a specific vertex/uniform type, start with [`DrawBuilder`].
If you need to freely mix different kinds of geometry (e.g., colored shapes and textured quads),
use [`DrawPool`], which manages multiple draw buffers internally and preserves drawing order.

See also: [`layout`] for utilities that assist in arranging 2D content.

## Tools

This module includes a set of higher-level "tools" for common drawing operations.
These tools work by appending geometry to a [`DrawBuilder`] using a shared *template*,
which defines non-positional vertex attributes like color or UVs.

### [`Pen`]

Draws stroked shapes using lines.

- Works with any vertex type.
- Uses a single template for all generated vertices.

- Used with [`ColorVertex`] and [`ColorTemplate`].

### [`Paint`]

Fills interior areas of shapes (e.g., rects, circles).

- Used with [`ColorVertex`] and [`ColorTemplate`].

### [`Sprite`]

Renders textured quads and sprites.

- Uses one template per corner to define UVs and color blending.
- Used with [`TexturedVertex`] and [`TexturedTemplate`].

### [`Scribe`]

Draws text using a [`FontResource`].

- Requires an atlas texture and font metrics.
- Uses [`TextVertex`] and [`TextTemplate`] internally.
