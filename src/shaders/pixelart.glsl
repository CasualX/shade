#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec4 v_color;
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
in vec2 a_uv;
in vec4 a_color;

uniform mat3x2 u_transform;
uniform vec4 u_colorModulation;

void main() {
	v_color = srgbToLinear(a_color) * u_colorModulation;
	v_uv = a_uv;

	vec2 pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;

uniform sampler2D u_texture;

vec2 pixelartSampleUV(sampler2D tex, vec2 uv) {
	vec2 size_texels = vec2(textureSize(tex, 0));
	vec2 texels = uv * size_texels;
	vec2 sample_texels;
	#ifdef PIXELART_CRISPY
		sample_texels = floor(texels) + 0.5;
	#else
		vec2 seam = floor(texels + 0.5);
		vec2 footprint = max(fwidth(texels), vec2(1e-6));
		sample_texels = seam + clamp((texels - seam) / footprint, -0.5, 0.5);
	#endif
	return sample_texels / size_texels;
}

void main() {
	vec4 color = texture(u_texture, pixelartSampleUV(u_texture, v_uv));
	o_fragColor = clamp(v_color * color, 0.0, 1.0);
}
#endif
