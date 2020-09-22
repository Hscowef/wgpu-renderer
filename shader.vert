#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec2 a_uv;

layout(location=0) out vec2 v_uv;


void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_pos, 1.0);
}