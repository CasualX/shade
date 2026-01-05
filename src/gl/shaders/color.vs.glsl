#version 330 core

in vec2 a_pos;
in vec4 a_color1;

out vec4 v_color;

uniform mat3x2 u_transform;
uniform vec4 u_colorModulation;

void main() {
	v_color = a_color1 * u_colorModulation;

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
