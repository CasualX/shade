#version 330 core

in vec2 a_pos;
in vec2 a_uv;
in vec4 a_color;
in vec4 a_outline;

out vec2 v_uv;
out vec4 v_color;
out vec4 v_outline;

uniform mat3x2 u_transform;
uniform float u_gamma;

void main() {
	v_uv = a_uv;
	v_color = pow(a_color, vec4(u_gamma));
	v_outline = pow(a_outline, vec4(u_gamma));

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
