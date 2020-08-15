#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

layout(location = 0) out VS_OUT
{
    vec3 normal;
} vs_out;

void main() {
    gl_Position = u_view_proj * vec4(i_position, 1.0);

    vs_out.normal = i_normal;
}