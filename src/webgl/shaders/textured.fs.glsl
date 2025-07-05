// WebGL GLSL ES 1.00

precision mediump float;

varying vec4 v_color;
varying vec2 v_uv;

uniform sampler2D u_texture;
uniform vec4 u_color_add;

void main() {
	vec4 texColor = texture2D(u_texture, v_uv);
	gl_FragColor = clamp(v_color * texColor + u_color_add, 0.0, 1.0);
}
