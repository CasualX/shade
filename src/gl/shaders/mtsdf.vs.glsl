#version 330 core

in vec2 a_pos;
in vec2 a_uv;
in vec4 a_color;
in vec4 a_outline;

out vec2 v_uv;
out vec4 v_color;
out vec4 v_outline;

uniform mat3x2 u_transform;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_uv = a_uv;
	v_color = srgbToLinear(a_color);
	v_outline = a_outline;

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
