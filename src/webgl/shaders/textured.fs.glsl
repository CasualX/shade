// WebGL GLSL ES 1.00

precision mediump float;

varying vec4 v_color;
varying vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	vec4 color = texture2D(u_texture, v_uv);
	gl_FragColor = clamp(v_color * color, 0.0, 1.0);
}
