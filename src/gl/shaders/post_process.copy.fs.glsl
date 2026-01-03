#version 330 core

in vec2 v_uv;

uniform sampler2D u_texture;

layout(location = 0) out vec4 o_fragColor;

void main() {
	o_fragColor = texture(u_texture, v_uv);
}
