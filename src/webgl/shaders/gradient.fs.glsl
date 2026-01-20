precision highp float;

const float PI = 3.14159265358979323846;
const float TAU = PI + PI;

varying vec4 v_color1;
varying vec4 v_color2;
varying vec2 v_uv;

#ifdef SOURCE_TEXTURE
uniform sampler2D u_texture;
#endif

// -- Compute Gradient Distance -----------------------------------
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

float fract(float x) {
	return x - floor(x);
}

vec4 get_gradient_color(float s) {
	#ifdef SOURCE_COLOR
		s = smoothstep(0.0, 1.0, s);
		return mix(v_color1, v_color2, s);
	#elif defined(SOURCE_TEXTURE)
		return texture2D(u_texture, vec2(0.5, s));
	#else
		return vec4(1.0, 0.0, 1.0, 1.0); // fallback pink
	#endif
}

void main() {
	float d = compute_distance(v_uv);
	float s = apply_repeat(d);
	gl_FragColor = get_gradient_color(s);
}
