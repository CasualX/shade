#version 330 core

// Vertex attributes
in vec2 a_pos;
in vec4 a_color1;
in vec4 a_color2;

// Varyings
out vec4 v_color1;
out vec4 v_color2;
out vec2 v_uv;
out vec2 v_pos;

// Uniforms
uniform mat3x2 u_transform;
uniform mat3x2 u_pattern;
uniform vec4 u_colorModulation;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	vec2 pos = u_transform * vec3(a_pos, 1.0);
	vec2 uv = u_pattern * vec3(a_pos, 1.0);

	gl_Position = vec4(pos, 0.0, 1.0);

	v_uv = uv;
	v_pos = pos;
	v_color1 = srgbToLinear(a_color1) * u_colorModulation;
	v_color2 = srgbToLinear(a_color2) * u_colorModulation;
}
