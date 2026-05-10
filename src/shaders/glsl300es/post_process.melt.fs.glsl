#version 300 es
precision highp float;

out vec4 o_fragColor;

in vec2 v_uv;

uniform sampler2D u_scene;
uniform sampler2D u_delays;
uniform float u_time;

void main() {
	float delay = texture(u_delays, vec2(v_uv.x, 0.5)).r;
	float offset = max(u_time - delay, 0.0);

	vec2 uv = v_uv;
	uv.y -= offset;

	if (uv.y < 0.0) {
		discard;
	}

	o_fragColor = texture(u_scene, uv);
}
