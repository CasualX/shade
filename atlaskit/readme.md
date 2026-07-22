# Atlaskit

Atlaskit packs named raster images, SVG shapes, and generated font atlases into a texture atlas for Shade.

A build produces two files from an INI manifest:

- `<output>.png` contains the atlas texture.
- `<output>.json` contains the atlas metadata, sprite properties, and font data. It is compact by default; pass `--pretty` for indented output.

SVG input is converted to a signed-distance field with [msdfgen](https://github.com/Chlumsky/msdfgen).  
TTF and OTF fonts are converted with [msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen).  
PNG, GIF, and JPEG input is copied into a bitmap atlas.

## Quick start: bitmap atlas

Create `sprites.ini` next to your images:

```ini
# Atlas-wide settings come first.
Margin=1

[Sprite:Player]
Path=images/player.png
Origin=16,24

[Sprite:Coin]
Path=images/coin.png

[Sprite:CoinFlipped]
Path=images/coin.png
Transform=FlipX
```

Install and run the builder:

```sh
cargo install --git https://github.com/CasualX/shade atlaskit
atlaskit build path/to/sprites.ini --output path/to/texture
```

Run the builder from the repository root:

```sh
cargo run -p atlaskit -- build path/to/sprites.ini --output path/to/texture
```

This writes `texture.png` and `texture.json`. Paths in the manifest are resolved
relative to the manifest itself, not the directory from which Atlaskit is run.

## Manifest structure

The manifest is an INI file with parts:

1. Atlas-wide properties at the top of the file.
2. `[Sprite:Name]` section for every exported sprite name.
3. `[Font:Name]` resource sections to embed fonts.

```ini
# Global defaults
Margin=1
MaxSize=4096

[Sprite:NameUsedByTheApplication]
Path=relative/or/absolute/path.png

[Sprite:AnotherName]
Path=another-image.png
Margin=2
Transform=Rotate90
Origin=8,12
```

Property names and most values are case-sensitive. Use the spelling shown in
this document. Whitespace around property names and values is ignored, and `#`
starts a comment. Do not repeat a property in the same scope. Resource names
must be non-empty. Repeating a sprite name creates an animation whose frames
follow manifest order.

Once the first resource section begins, all following properties belong to a sprite or font.
Atlas-wide properties therefore cannot appear between or after resource sections.

### Atlas properties

| Property | Values | Default | Meaning |
| --- | --- | --- | --- |
| `Width` | integer > 0 | automatic | Fixes the output width. If omitted, the packer chooses it. |
| `Height` | integer > 0 | automatic | Fixes the output height. If omitted, the packer chooses it. |
| `MaxSize` | integer > 0 | `4096` | Maximum permitted width and height while finding a packing. Ignored when both dimensions are fixed. |
| `Kind` | `bitmap`, `sdf`, `psdf`, `msdf`, `mtsdf` | `bitmap` | Texture encoding written to the atlas metadata. |
| `DistanceRange` | number >= 0 | `0` | Representable signed-distance range written to the atlas metadata, in atlas pixels. |
| `Margin` | integer >= 0 | `1` | Empty/gutter pixels reserved on every side of each sprite. A sprite can override it. |
| `RecoverAlphaColors` | `true`, `false` | `false` | Recovers RGB values in fully transparent atlas pixels from neighboring opaque pixels before writing the PNG. |
| `Processor` | `blit`, `msdfgen` | inferred | Forces the default input processor. A sprite can override it. |
| `Blit.Gutter` | `ClampToEdge`, `SelfTiled`, `Transparent` | `ClampToEdge` | Selects how bitmap sprite margins are filled. A sprite can override it. |
| `Msdfgen.Mode` | `sdf`, `psdf`, `msdf`, `mtsdf` | `mtsdf` | Selects the kind of distance field generated for SVGs. It does not select the sprite processor. |
| `Msdfgen.Range` | number > 0 | `4` | Signed-distance range passed to msdfgen, in output pixels. |
| `Msdfgen.Size` | number > 0 | `32` | Default SVG scale. A sprite can override it. |
| `Msdfgen.Autoframe` | `true`, `false` | `false` | Lets msdfgen frame the SVG shape automatically. A sprite can override it. |
| `Msdfgen.YFlip` | `true`, `false` | `false` | Requests msdfgen's vertical-axis flip. A sprite can override it. |
| `Msdfgen.Overlap` | `true`, `false` | `false` | Enables msdfgen overlap support. A sprite can override it. |
| `Msdfgen.FillRule` | `nonzero`, `evenodd`, `positive`, `negative` | `nonzero` | Fill rule used to interpret SVG contours. A sprite can override it. |
| `MsdfAtlasGen.Mode` | `sdf`, `psdf`, `msdf`, `mtsdf` | `mtsdf` | Selects the kind of distance field generated for fonts. |
| `MsdfAtlasGen.Range` | number > 0 | `4` | Signed-distance range passed to msdf-atlas-gen, in output pixels. |
| `MsdfAtlasGen.Size` | number > 0 | `32` | Default font size in output pixels per em. A font can override it. |
| `MsdfAtlasGen.OuterPadding` | number >= 0 | `1` | Default extra pixels around generated glyphs. A font can override it. |
| `MsdfAtlasGen.Overlap` | `true`, `false` | `false` | Enables overlap-aware font generation. A font can override it. |

`Width` and `Height` can be used independently. If one dimension is fixed, Atlaskit grows only the other dimension as needed.
If both are fixed, `MaxSize` is ignored and the build fails only when the sprites do not fit the requested dimensions.

When `Processor` is omitted, a path ending in `.svg` uses `msdfgen`; all other paths use `blit`.
Setting a root `Processor` replaces this inference for every sprite that does not override it.
There is no `auto` value for restoring inference inside a sprite.

`Kind` and `DistanceRange` describe the final texture independently of how its inputs are generated.
Atlaskit does not derive atlas metadata from either generator's mode or range.

`Msdfgen.Mode` describes how SVGs selected by `Processor` or their `.svg` extension are generated.
`MsdfAtlasGen.*` settings are independent and apply only to `[Font:*]` resources.

### Sprite properties

| Property | Required | Meaning |
| --- | --- | --- |
| `Path` | yes | Source file. Relative paths are resolved from the manifest directory. |
| `Duration` | for animations | Frame duration in seconds, as a number or float ratio such as `2/60`. Required on every repeated sprite name and forbidden on a name that occurs only once. |
| `Processor` | no | Overrides the atlas processor with `blit` or `msdfgen`. |
| `Margin` | no | Overrides the atlas margin. |
| `Blit.Gutter` | no | Overrides the atlas bitmap gutter mode. It is ignored by `msdfgen`. |
| `Transform` | no | UV transform stored in the JSON metadata; see below. Defaults to `None`. |
| `Origin` | no | `X,Y` pixel coordinate relative to the sprite's top-left corner. Defaults to `0,0`. |
| `Msdfgen.Size` | no | Overrides the atlas SVG size. |
| `Msdfgen.Autoframe` | no | Overrides the atlas autoframe setting. |
| `Msdfgen.YFlip` | no | Overrides the atlas Y-flip setting. |
| `Msdfgen.Overlap` | no | Overrides the atlas overlap setting. |
| `Msdfgen.FillRule` | no | Overrides the atlas SVG fill rule. |

The canonical `Transform` values are:

| Value | Effect |
| --- | --- |
| `None` | No transform. |
| `Rotate90` | Rotate 90 degrees clockwise. |
| `Rotate180` | Rotate 180 degrees. |
| `Rotate270` | Rotate 270 degrees clockwise. |
| `FlipX` | Flip horizontally. |
| `FlipY` | Flip vertically. |
| `FlipSlash` | Flip across the `/` diagonal. |
| `FlipBackslash` | Flip across the `\` diagonal. |

Transforms do not alter the packed pixels. They tell Shade how to transform the
sprite's UV coordinates while drawing. Consequently, several sprite names can
reuse one source image with different transforms without packing duplicate
pixels. `Origin` is also metadata; it identifies the sprite's logical anchor.

Repeat a sprite section to create an animation. Every frame must specify its
own `Duration`, and the emitted atlas sprite contains the frames in manifest
order:

```ini
[Sprite:Coin]
Path=coin-1.png
Duration=0.1

[Sprite:Coin]
Path=coin-2.png
Duration=0.15
```

## SVG distance-field atlas

SVG sprites require an msdfgen executable.
Each SVG must have an `<svg>` element with a valid `viewBox` containing four
finite numbers and a positive width and height.

```ini
Margin=1
MaxSize=2048
Kind=mtsdf
DistanceRange=4

Msdfgen.Mode=mtsdf
Msdfgen.Range=4
Msdfgen.Size=32
Msdfgen.Autoframe=false
Msdfgen.YFlip=false
Msdfgen.Overlap=false
Msdfgen.FillRule=nonzero

[Sprite:ArrowUp]
Path=icons/arrow-up.svg

# Reuses the generated arrow pixels and changes only its UV transform.
[Sprite:ArrowDown]
Path=icons/arrow-up.svg
Transform=Rotate180

[Sprite:Logo]
Path=icons/logo.svg
Msdfgen.Size=64
Msdfgen.FillRule=evenodd
```

Build it by passing the msdfgen program on the command line:

```sh
cargo run -p atlaskit -- build icons.ini \
  --output generated/icons \
  --msdfgen path/to/msdfgen.exe \
  --preview
```

On Linux, Atlaskit automatically runs an msdfgen path ending in `.exe` through
Wine. `--preview` additionally writes `generated/icons.preview.png`, which makes
a distance-field atlas easier to inspect visually.

With autoframe disabled, SVG dimensions come from the view box and
`Msdfgen.Size`: the output size before margins is
`ceil(viewBox size * Msdfgen.Size / 32)`. The view-box coordinate mapping is
preserved. With autoframe enabled, msdfgen frames the shape within those output
dimensions. Autoframed sprites must use `Margin=0`; Atlaskit rejects a nonzero
margin because autoframe would fit the shape into the gutter as well as the
logical sprite rectangle.

## Input processing and margins

The `blit` processor accepts lowercase `.png`, `.gif`, `.jpg`, and `.jpeg` paths.
It cannot copy SVG files. `Blit.Gutter=ClampToEdge` extends the source image's
edge pixels into the margin, `Blit.Gutter=SelfTiled` wraps pixels from the
opposite edge, and `Blit.Gutter=Transparent` fills the margin with transparent
black.

The `msdfgen` processor accepts `.svg` paths only. Its margin consists
of distance-field pixels rendered by msdfgen rather than duplicated border pixels,
so the `Blit.Gutter` property is ignored for these sprites.

Atlaskit deduplicates sprites that have the same resolved path, processor
settings, and margin. Bitmap gutter mode is part of those processor settings.
`Transform` and `Origin` may differ without causing a second copy of the pixels
to be packed.

## Font atlas resources

A font resource needs a TTF or OTF `Path`. It uses ASCII by default, so the smallest useful manifest is:

```ini
Kind=mtsdf
DistanceRange=4

MsdfAtlasGen.Mode=mtsdf
MsdfAtlasGen.Range=4

[Font:Ui]
Path=fonts/ui.ttf
MsdfAtlasGen.Size=32
```

Build it by passing the generator program:

```sh
cargo run -p atlaskit -- build atlas.ini \
  --output generated/atlas \
  --msdf-atlas-gen path/to/msdf-atlas-gen.exe
```

On Linux, a program path ending in `.exe` is run through Wine. Atlaskit asks
msdf-atlas-gen for a power-of-two rectangular PNG and JSON document, parses the
document through `shade::msdfgen`, extracts every drawable glyph rectangle,
packs each glyph with the other resources, and records its new atlas position.

### Font properties

| Property | Default | Meaning |
| --- | --- | --- |
| `Path` | required | TTF or OTF input. Relative paths are resolved from the manifest. |
| `MsdfAtlasGen.Charset` | ASCII | Path passed directly to msdf-atlas-gen as its charset file. |
| `MsdfAtlasGen.Chars` | ASCII | Inline character-set expression passed directly to msdf-atlas-gen. |
| `FontScale` | `1` | Scale this input face's glyph geometry. |
| `MsdfAtlasGen.Size` | atlas setting or `32` | Font size in output pixels per em. |
| `MsdfAtlasGen.OuterPadding` | atlas setting or `1` | Extra pixels around each generated glyph. |
| `Margin` | atlas `Margin` or `1` | Gutter around each glyph when it is repacked. |
| `MsdfAtlasGen.Overlap` | atlas setting or `false` | Enable overlap-aware distance-field generation for this font. |

`MsdfAtlasGen.Charset` and `MsdfAtlasGen.Chars` are mutually exclusive. If
neither is present, msdf-atlas-gen's ASCII default is used. Charset files use the upstream
[character-set syntax](https://github.com/Chlumsky/msdf-atlas-gen#character-set-specification-syntax),
including ranges such as `[0x20, 0x7e]` and quoted UTF-8 strings.

Repeat a section name to combine fallback faces into one exported font. Each
section becomes an msdf-atlas-gen input separated by `-and`, while
`MsdfAtlasGen.Charset`, `MsdfAtlasGen.Chars`, and `FontScale` remain specific to
that face:

```ini
[Font:Ui]
Path=fonts/Adventure.otf
MsdfAtlasGen.Charset=charset.txt
MsdfAtlasGen.Size=32

[Font:Ui]
Path=fonts/segoe-ui-symbol.ttf
MsdfAtlasGen.Charset=symbols.txt
MsdfAtlasGen.Size=32
```

Repeated sections must agree on `MsdfAtlasGen.Size`, `MsdfAtlasGen.OuterPadding`,
`Margin`, and `MsdfAtlasGen.Overlap`.
If faces contain the same codepoint, the last face wins in the exported lookup table.

## Command-line options

```text
atlaskit build <MANIFEST> --output <OUTPUT> [OPTIONS]
atlaskit info <ATLAS>

Options:
  -o, --output <OUTPUT>      Prefix for the generated .png and .json files
      --msdfgen <PROGRAM>         msdfgen executable required by SVG sprites
      --msdf-atlas-gen <PROGRAM>  msdf-atlas-gen executable required by fonts
      --pretty                    Format the output JSON with indentation
      --preview                   Write <output>.preview.png
      --temp-dir <DIR>            New directory for intermediate generated images
      --keep-intermediate         Keep intermediate images after a successful build
```

`atlaskit info` summarizes an existing atlas JSON file, including its dimensions,
encoding, sprite and font counts, frame and glyph counts, unique regions, file
size, and the percentage of atlas pixels covered by drawable content. Coverage is
calculated from the union of sprite-frame and visible-glyph rectangles, so aliases
and overlapping animation frames are not counted more than once.

The directory supplied to `--temp-dir` must not already exist. When SVGs or fonts need processing and no directory is supplied, Atlaskit creates a temporary one.
Intermediate files are normally removed after a successful build and retained for inspection after a failed build.

For the complete generated CLI help, run:

```sh
cargo run -p atlaskit -- build --help
```
