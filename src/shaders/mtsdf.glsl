#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec2 v_uv;
VARYING vec4 v_color;
VARYING vec4 v_outline;

vec3 srgbToLinear(vec3 c) {
	#ifdef GLSL_CORE
		return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
	#else
		return c;
	#endif
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

#ifdef VERTEX_SHADER
in vec2 a_pos;
in vec2 a_uv;
in vec4 a_color;
in vec4 a_outline;

uniform mat3x2 u_transform;

void main() {
	v_uv = a_uv;
	v_color = srgbToLinear(a_color);
	v_outline = a_outline;

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;

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
	vec4 distances = texture(u_texture, v_uv);
	float d_sdf = median(distances.rgb);

	float width = screen_px_range();
	if (d_sdf <= 0.0)
		discard;

	float inner = width * (d_sdf - u_threshold) + 0.5 + u_outBias;
	float outer = width * (d_sdf - u_threshold + u_outlineWidthRelative) + 0.5 + u_outBias + u_outlineWidthAbsolute;

	inner = clamp(inner, 0.0, 1.0);
	outer = clamp(outer, 0.0, 1.0);

	vec4 color = v_color * inner + v_outline * (outer - inner);
	o_fragColor = color;
}
#endif
