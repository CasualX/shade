// WebGL GLSL ES 1.00

precision mediump float;

attribute vec2 a_pos;
attribute vec2 a_uv;
attribute vec4 a_color;
attribute vec4 a_outline;

uniform mat3 u_transform;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_outline;

void main() {
	v_uv = a_uv;
	v_color = a_color;
	v_outline = a_outline;

	vec3 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos.xy, 0.0, 1.0);
}
