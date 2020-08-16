#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_tex_coord;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_model;
    mat4 u_view_proj;
};

layout(location = 0) out VS_OUT
{
    vec3 normal;
    vec2 tex_coord;
    vec3 world_pos;
} vs_out;

void main() {

    vec4 position = vec4(i_position, 1.0);

    gl_Position = u_view_proj * u_model * position;

    vs_out.normal = i_normal;
    vs_out.tex_coord = i_tex_coord;
    vs_out.world_pos = (u_model * position).xyz; 
}