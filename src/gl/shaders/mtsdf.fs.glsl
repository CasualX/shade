#version 330 core

in vec2 v_uv;
in vec4 v_color;
in vec4 v_outline;

layout(location = 0) out vec4 o_fragcolor;

uniform sampler2D u_texture;
uniform vec2 u_unit_range;
uniform float u_width;
uniform float u_threshold;
uniform float u_out_bias;
uniform float u_outline_width_absolute;
uniform float u_outline_width_relative;
uniform float u_gamma;

float median(vec3 distances) {
	return max(min(distances.r, distances.g), min(max(distances.r, distances.g), distances.b));
}

float screen_px_range() {
	vec2 screenTexSize = vec2(1.0) / fwidth(v_uv);
	return max(0.5 * dot(u_unit_range, screenTexSize), 1.0);
}

void main() {
	vec4 distances = texture(u_texture, v_uv);
	float d_sdf = median(distances.rgb);

	float width = screen_px_range();

	// Discard fragments outside the glyph's encoded distance field (fully transparent areas)
	if (d_sdf <= 0.0)
		discard;

	float inner = width * (d_sdf - u_threshold) + 0.5 + u_out_bias;
	float outer = width * (d_sdf - u_threshold + u_outline_width_relative) + 0.5 + u_out_bias + u_outline_width_absolute;

	inner = clamp(inner, 0.0, 1.0);
	outer = clamp(outer, 0.0, 1.0);

	vec4 color = v_color * inner + v_outline * (outer - inner);
	o_fragcolor = pow(color, vec4(1.0 / u_gamma));
}
