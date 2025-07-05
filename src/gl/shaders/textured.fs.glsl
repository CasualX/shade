#version 330 core

in vec4 v_color;
in vec2 v_uv;

layout(location = 0) out vec4 o_color;

uniform sampler2D u_texture;
uniform vec4 u_color_add;

void main() {
	vec4 tex_color = texture(u_texture, v_uv);
	o_color = clamp(v_color * tex_color + u_color_add, 0.0, 1.0);
}
