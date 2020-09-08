#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec4 i_tangent;
layout(location = 3) in vec2 i_tex_coord;

layout(set=0, binding=0)
uniform Transforms {
    mat4 proj;
    mat4 view;
} u_camera;

layout(location = 0)
out VS_OUT {
    vec2 tex_coord;
    vec3 pos;
} vs_out;

void main()
{
    vs_out.pos = i_position; 
    vs_out.tex_coord = i_tex_coord;

    gl_Position =  u_camera.proj * u_camera.view * vec4(i_position, 1.0);
}