// WebGL GLSL ES 1.00

precision mediump float;

attribute vec2 a_pos;
attribute vec4 a_color1;

uniform mat3 u_transform;
uniform vec4 u_colorModulation;

varying vec4 v_color;

void main() {
	v_color = a_color1 * u_colorModulation;

	vec3 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos.xy, 0.0, 1.0);
}
