#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;

out vec2 texture_coord;

void main() {
    texture_coord = in_texture_coord;
    gl_Position = vec4(in_position, 1.0);
}