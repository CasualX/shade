// WebGL GLSL ES 1.00

precision highp float;

varying vec2 v_uv;

uniform sampler2D u_scene;
uniform sampler2D u_delays; // 1D delay texture stored as 1×N or N×1
uniform float u_time;

void main() {
	// Sample the precomputed delay from the texture
	float delay = texture2D(u_delays, vec2(v_uv.x, 0.5)).r;

	// Apply the DOOM melt offset
	float offset = max(u_time - delay, 0.0);

	vec2 uv = v_uv;
	uv.y -= offset;

	if (uv.y < 0.0) {
		discard;
	}

	gl_FragColor = texture2D(u_scene, uv);
}
