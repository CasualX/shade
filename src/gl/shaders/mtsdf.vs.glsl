#version 330 core
layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec2 a_texcoord;
layout (location = 2) in vec4 a_color;
layout (location = 3) in vec4 a_outline;

out vec2 v_texcoord;
out vec4 v_color;
out vec4 v_outline;

uniform mat3x2 u_transform;
uniform float u_gamma;

void main() {
	v_texcoord = a_texcoord;
	v_color = pow(a_color, vec4(u_gamma));
	v_outline = pow(a_outline, vec4(u_gamma));
	gl_Position = vec4(u_transform * vec3(a_pos, 1.0), 0.0, 1.0);
}
