precision mediump float;

// Vertex attributes
attribute vec2 a_pos;
attribute vec4 a_color1;
attribute vec4 a_color2;

// Varyings
varying vec4 v_color1;
varying vec4 v_color2;
varying vec2 v_uv;
varying vec2 v_pos;

// Uniforms
uniform mat3 u_transform;
uniform mat3 u_pattern;
uniform vec4 u_colorModulation;

void main() {
	vec3 pos = u_transform * vec3(a_pos, 1.0);
	vec3 uv = u_pattern * vec3(a_pos, 1.0);

	gl_Position = vec4(pos.xy, 0.0, 1.0);

	v_uv = uv.xy;
	v_pos = pos.xy;
	v_color1 = a_color1 * u_colorModulation;
	v_color2 = a_color2 * u_colorModulation;
}
