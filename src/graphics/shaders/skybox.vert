#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec4 i_tangent;
layout(location = 3) in vec2 i_tex_coord;

layout(set=0, binding=0)
uniform Transforms {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 world_pos;
} u_camera;

layout(location = 0)
out VS_OUT {
    vec3 local_pos;
} vs_out;

void main() {

    vs_out.local_pos = i_position;

    mat4 rotView = mat4(mat3(u_camera.view)); // remove translation from the view matrix
    vec4 clipPos = u_camera.proj * rotView * vec4(vs_out.local_pos, 1.0);

    gl_Position = clipPos.xyww;
}