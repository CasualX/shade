#version 330 core

out vec4 o_fragColor;

in vec4 v_color;

void main() {
	o_fragColor = clamp(v_color, 0.0, 1.0);
}
