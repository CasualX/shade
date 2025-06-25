// WebGL GLSL ES 1.00

precision mediump float;

attribute vec2 a_pos;
attribute vec2 a_texcoord;
attribute vec4 a_color;
attribute vec4 a_outline;

uniform vec3 u_transform[2];
uniform float u_gamma;

varying vec2 v_texcoord;
varying vec4 v_color;
varying vec4 v_outline;

void main() {
	v_texcoord = a_texcoord;
	v_color = pow(a_color, vec4(u_gamma));
	v_outline = pow(a_outline, vec4(u_gamma));

	// Multiply row-major 2x3 matrix by vec3(a_pos, 1.0)
	float x = dot(vec3(a_pos, 1.0), u_transform[0]);
	float y = dot(vec3(a_pos, 1.0), u_transform[1]);

	gl_Position = vec4(x, y, 0.0, 1.0);
}
