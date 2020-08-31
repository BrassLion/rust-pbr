#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec4 i_tangent;
layout(location = 3) in vec2 i_tex_coord;

layout(location = 0) out vec3 f_pos;

layout(set=0, binding=0)
uniform Transforms {
    mat4 proj;
    mat4 view;
} u_camera;

void main()
{
    f_pos = i_position; 

    gl_Position =  u_camera.proj * u_camera.view * vec4(i_position, 1.0);
}