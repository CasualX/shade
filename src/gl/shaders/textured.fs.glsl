#version 330 core

out vec4 o_fragColor;

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	vec4 color = texture(u_texture, v_uv);
	o_fragColor = clamp(v_color * color, 0.0, 1.0);
}
