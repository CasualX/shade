#version 330 core

in vec4 v_color;

layout(location = 0) out vec4 o_color;

void main() {
	o_color = clamp(v_color, 0.0, 1.0);
}
