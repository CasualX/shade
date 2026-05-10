#version 330 core

out vec4 o_fragColor;

in vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	o_fragColor = texture(u_texture, v_uv);
}
