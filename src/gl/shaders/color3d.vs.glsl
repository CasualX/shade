#version 330 core

in vec3 a_pos;
in vec4 a_color;

out vec4 v_color;

uniform mat4 u_transform;
uniform vec4 u_color;
uniform vec4 u_add_color;

void main() {
	v_color = clamp(a_color * u_color + u_add_color, 0.0, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
