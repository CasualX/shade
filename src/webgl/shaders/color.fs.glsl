// WebGL GLSL ES 1.00

precision mediump float;

varying vec4 v_color;

void main() {
	gl_FragColor = clamp(v_color, 0.0, 1.0);
}
