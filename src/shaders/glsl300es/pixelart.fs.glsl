#version 300 es
precision highp float;

out vec4 o_fragColor;

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	vec2 texels = v_uv * vec2(textureSize(u_texture, 0));
	vec2 sample_texels;
	#ifdef PIXELART_CRISPY
		sample_texels = floor(texels) + 0.5;
	#else
		vec2 seam = floor(texels + 0.5);
		vec2 footprint = max(fwidth(texels), vec2(1e-6));
		sample_texels = seam + clamp((texels - seam) / footprint, -0.5, 0.5);
	#endif
	vec4 color = texture(u_texture, sample_texels / vec2(textureSize(u_texture, 0)));
	o_fragColor = clamp(v_color * color, 0.0, 1.0);
}
