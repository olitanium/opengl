#version 330 core

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_texture_coord;
layout (location = 2) in vec3 in_normal;

out vec2 texture_coord;
out vec3 frag_normal;
out vec3 frag_position;
out vec4 screen_pos;

uniform mat4 model;
uniform mat4 projtimesview;

void main() {
    texture_coord = in_texture_coord;
    
    mat3 normal_matrix = mat3(transpose(inverse(model)));
    frag_normal = normal_matrix * in_normal;
    //frag_normal = in_normal;

    vec4 model_position = (model * vec4(in_position, 1.0));
    frag_position = model_position.xyz;
    screen_pos = projtimesview * model_position;
    gl_Position = screen_pos;
}