#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec2 v_uv;

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

#ifdef VERTEX_SHADER
in vec2 a_pos;
in vec2 a_uv;

void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;

uniform sampler2D u_texture;

void main() {
	o_fragColor = texture(u_texture, pixelartSampleUV(u_texture, v_uv));
}
#endif
