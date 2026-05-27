#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec4 v_color;

vec3 srgbToLinear(vec3 c) {
	#ifdef GLSL_CORE
		return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
	#else
		return c;
	#endif
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

#ifdef VERTEX_SHADER
in vec3 a_pos;
in vec4 a_color;

uniform mat4 u_transform;
uniform vec4 u_colorModulation;

void main() {
	v_color = srgbToLinear(a_color) * u_colorModulation;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;

void main() {
	o_fragColor = clamp(v_color, 0.0, 1.0);
}
#endif
