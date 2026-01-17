// WebGL GLSL ES 1.00
#extension GL_OES_standard_derivatives : enable

precision mediump float;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_outline;

uniform sampler2D u_texture;
uniform vec2 u_unitRange;
uniform float u_width;
uniform float u_threshold;
uniform float u_outBias;
uniform float u_outlineWidthAbsolute;
uniform float u_outlineWidthRelative;

float median(vec3 distances) {
	return max(min(distances.r, distances.g), min(max(distances.r, distances.g), distances.b));
}

float screen_px_range() {
	vec2 screenTexSize = vec2(1.0) / fwidth(v_uv);
	return max(0.5 * dot(u_unitRange, screenTexSize), 1.0);
}

void main() {
	vec4 distances = texture2D(u_texture, v_uv);
	float d_sdf = median(distances.rgb);

	float width = screen_px_range();

	// Discard fragments outside the glyph's encoded distance field (fully transparent areas)
	if (d_sdf <= 0.0)
		discard;

	float inner = width * (d_sdf - u_threshold) + 0.5 + u_outBias;
	float outer = width * (d_sdf - u_threshold + u_outlineWidthRelative) + 0.5 + u_outBias + u_outlineWidthAbsolute;

	inner = clamp(inner, 0.0, 1.0);
	outer = clamp(outer, 0.0, 1.0);

	vec4 color = v_color * inner + v_outline * (outer - inner);
	gl_FragColor = color;
}
