// WebGL GLSL ES 1.00

precision highp float;

attribute vec2 a_pos;
attribute vec4 a_color1;

uniform mat3 u_transform;
uniform vec4 u_colorModulation;

varying vec4 v_color;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_color = srgbToLinear(a_color1) * u_colorModulation;

	vec3 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos.xy, 0.0, 1.0);
}
