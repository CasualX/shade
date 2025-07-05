#version 330 core

in vec3 a_pos;
in vec4 a_color;

out vec4 v_color;

uniform mat4 u_transform;
uniform vec4 u_colormod;
uniform vec4 u_color_add;

void main() {
	v_color = a_color * u_colormod + u_color_add;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
