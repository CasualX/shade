Multi-channel SDF based fonts
=============================

This document describes how to use [msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen) to prerasterize fonts into textures to use to render crisp text.

Download the latest release from their [releases](https://github.com/Chlumsky/msdf-atlas-gen/releases) page. The `msdf-atlas-gen.exe` works just fine on Linux with Wine.

Bring a font of your choosing, for example use [Roboto](https://fonts.google.com/specimen/Roboto) from Google Fonts.

The run the following command (it can be customized to your needs):

```sh
msdf-atlas-gen.exe -font Roboto-Regular.ttf -chars "[0x20, 0x7e]" -type mtsdf -format png -imageout Roboto.png -json Roboto.json -pxpadding 4
```

Results in two files: the texture `Roboto.png` and the metadata `Roboto.json`.

```rust
let mut g: &mut shade::Graphics = ..;

let roboto = {
	let font = std::fs::read_to_string("Roboto.json").unwrap();
	let font: shade::msdfgen::Font = serde_json::from_str(&font).unwrap();

	let texture = shade::png::load(g, Some("fonts/Roboto"), "Roboto.png", &shade::png::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	shade::d2::Font { font, texture, shader}
};
```

Parse the font, load the texture, compile the shader and wrap it all up in a `FontResource` instance.

```rust
let mut cv = shade::d2::TextBuffer::new();
cv.blend_mode = shade::BlendMode::Alpha;

let mut scribe = shade::d2::Scribe {
	font_size: 64.0,
	line_height: 64.0,
	..Default::default()
};

let mut pos = cvmath::Vec2(0.0, 0.0);
cv.text_write(&roboto, &scribe, &mut pos, "Hello, World");

cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
```
