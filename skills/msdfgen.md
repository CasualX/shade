Multi-channel signed distance field generator by Viktor Chlumsky v1.13.0 with Skia
----------------------------------------------------------------------------------
  Usage: msdfgen.exe <mode> <input specification> <options>

MODES
  sdf - Generate conventional monochrome (true) signed distance field.
  psdf - Generate monochrome signed perpendicular distance field.
  msdf - Generate multi-channel signed distance field. This is used by default if no mode is specified.
  mtsdf - Generate combined multi-channel and true signed distance field in the alpha channel.
  metrics - Report shape metrics only.

INPUT SPECIFICATION
  -defineshape <definition>
        Defines input shape using the ad-hoc text definition.
  -font <filename.ttf> <character code>
        Loads a single glyph from the specified font file.
        Format of character code is '?', 63, 0x3F (Unicode value), or g34 (glyph index).
  -shapedesc <filename.txt>
        Loads text shape description from a file.
  -stdin
        Reads text shape description from the standard input.
  -svg <filename.svg>
        Loads the last vector path found in the specified SVG file.
  -varfont <filename and variables> <character code>
        Loads a single glyph from a variable font. Specify variable values as x.ttf?var1=0.5&var2=1

OPTIONS
  -angle <angle>
        Specifies the minimum angle between adjacent edges to be considered a corner. Append D for degrees.
  -apxrange <outermost distance> <innermost distance>
        Specifies the outermost (negative) and innermost representable distance in pixels.
  -arange <outermost distance> <innermost distance>
        Specifies the outermost (negative) and innermost representable distance in shape units.
  -ascale <x scale> <y scale>
        Sets the scale used to convert shape units to pixels asymmetrically.
  -autoframe
        Automatically scales (unless specified) and translates the shape to fit.
  -coloringstrategy <simple / inktrap / distance>
        Selects the strategy of the edge coloring heuristic.
  -dimensions <width> <height>
        Sets the dimensions of the output image.
  -edgecolors <sequence>
        Overrides automatic edge coloring with the specified color sequence.
  -emnormalize
        Before applying scale, normalizes font glyph coordinates so that 1 = 1 em.
  -errorcorrection <mode>
        Changes the MSDF/MTSDF error correction mode. Use -errorcorrection help for a list of valid modes.
  -errordeviationratio <ratio>
        Sets the minimum ratio between the actual and maximum expected distance delta to be considered an error.
  -errorimproveratio <ratio>
        Sets the minimum ratio between the pre-correction distance error and the post-correction distance error.
  -estimateerror
        Computes and prints the distance field's estimated fill error to the standard output.
  -exportshape <filename.txt>
        Saves the shape description into a text file that can be edited and loaded using -shapedesc.
  -exportsvg <filename.svg>
        Saves the shape geometry into a simple SVG file.
  -fillrule <nonzero / evenodd / positive / negative>
        Sets the fill rule for the scanline pass. Default is nonzero.
  -format <png / bmp / tiff / rgba / fl32 / text / textfloat / bin / binfloat / binfloatbe>
        Specifies the output format of the distance field. Otherwise it is chosen based on output file extension.
  -guesswinding
        Attempts to detect if shape contours have the wrong winding and generates the SDF with the right one.
  -help
        Displays this help.
  -legacy
        Uses the original (legacy) distance field algorithms.
  -noemnormalize
        Raw integer font glyph coordinates will be used. Without this option, legacy scaling will be applied.
  -nopreprocess
        Disables path preprocessing which resolves self-intersections and overlapping contours.
  -o <filename>
        Sets the output file name. The default value is "output.png".
  -overlap
        Switches to distance field generator with support for overlapping contours.
  -printmetrics
        Prints relevant metrics of the shape to the standard output.
  -pxrange <range>
        Sets the width of the range between the lowest and highest signed distance in pixels.
  -range <range>
        Sets the width of the range between the lowest and highest signed distance in shape units.
  -reversewinding
        Generates the distance field as if the shape's vertices were in reverse order.
  -scale <scale>
        Sets the scale used to convert shape units to pixels.
  -scanline
        Performs an additional scanline pass to fix the signs of the distances.
  -seed <n>
        Sets the random seed for edge coloring heuristic.
  -stdout
        Prints the output instead of storing it in a file. Only text formats are supported.
  -testrender <filename.png> <width> <height>
        Renders an image preview using the generated distance field and saves it as a PNG file.
  -testrendermulti <filename.png> <width> <height>
        Renders an image preview without resolving the color channels.
  -translate <x> <y>
        Sets the translation of the shape in shape units.
  -version
        Prints the version of the program.
  -windingpreprocess
        Attempts to fix only the contour windings assuming no self-intersections and even-odd fill rule.
  -yflip
        Inverts the Y-axis in the output distance field. The default orientation is upward.

