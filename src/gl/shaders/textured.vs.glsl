#version 330 core

in vec2 a_pos;
in vec2 a_uv;
in vec4 a_color;

out vec4 v_color;
out vec2 v_uv;

uniform mat3x2 u_transform;
uniform vec4 u_colormod;

void main() {
	v_color = a_color * u_colormod;
	v_uv = a_uv;

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
