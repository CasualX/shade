#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec4 v_color1;
VARYING vec4 v_color2;
VARYING vec2 v_uv;

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
in vec2 a_pos;
in vec4 a_color1;
in vec4 a_color2;

uniform mat3x2 u_transform;
uniform mat3x2 u_pattern;
uniform vec4 u_colorModulation;

void main() {
	vec2 pos = u_transform * vec3(a_pos, 1.0);
	v_uv = u_pattern * vec3(a_pos, 1.0);

	gl_Position = vec4(pos, 0.0, 1.0);
	v_color1 = srgbToLinear(a_color1) * u_colorModulation;
	v_color2 = srgbToLinear(a_color2) * u_colorModulation;
}
#endif

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;

const float PI = 3.14159265358979323846;
const float TAU = PI + PI;

#ifdef SOURCE_TEXTURE
uniform sampler2D u_texture;
#endif

float compute_distance(vec2 uv) {
	#ifdef SHAPE_LINEAR
		return uv.x;
	#elif defined(SHAPE_RADIAL)
		return length(uv);
	#elif defined(SHAPE_SQUARE)
		return abs(uv.x) + abs(uv.y);
	#elif defined(SHAPE_SCONE)
		return abs(atan(uv.x, uv.y)) / PI;
	#elif defined(SHAPE_ACONE)
		return (atan(uv.x, uv.y) / TAU) + 0.5;
	#elif defined(SHAPE_SPIRAL)
		return length(uv) + ((atan(uv.x, uv.y) / TAU) + 0.5);
	#else
		return 0.0;
	#endif
}

float apply_repeat(float d) {
	#ifdef REPEAT_ONE
		return clamp(d, 0.0, 1.0);
	#elif defined(REPEAT_MIRROR)
		return abs(fract(d) - 0.5) * 2.0;
	#elif defined(REPEAT_WRAP)
		return fract(d);
	#elif defined(REPEAT_BI)
		return clamp(abs(d - 1.0), 0.0, 1.0);
	#else
		return d;
	#endif
}

vec4 get_gradient_color(float s) {
	#ifdef SOURCE_COLOR
		s = smoothstep(0.0, 1.0, s);
		return mix(v_color1, v_color2, s);
	#elif defined(SOURCE_TEXTURE)
		return texture(u_texture, vec2(0.5, s));
	#else
		return vec4(1.0, 0.0, 1.0, 1.0);
	#endif
}

void main() {
	float d = compute_distance(v_uv);
	float s = apply_repeat(d);
	o_fragColor = get_gradient_color(s);
}
#endif
