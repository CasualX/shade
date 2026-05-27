#version unified 330 core, 300 es

#ifdef GLSL_ES
precision highp float;
#endif

VARYING vec2 v_uv;

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
	o_fragColor = texture(u_texture, v_uv);
}
#endif
