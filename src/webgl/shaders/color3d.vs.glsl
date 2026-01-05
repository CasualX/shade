// WebGL GLSL ES 1.00

precision mediump float;

attribute vec3 a_pos;
attribute vec4 a_color;

uniform mat4 u_transform;
uniform vec4 u_colorModulation;

varying vec4 v_color;

void main() {
	v_color = a_color * u_colorModulation;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
