#version 450

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec4 i_tangent;
layout(location = 3) in vec2 i_tex_coord;

layout(set=0, binding=0)
uniform Transforms {
    mat4 u_model;
    mat4 u_view_proj;
    vec3 u_camera_world_position;
};

layout(location = 0)
out VS_OUT {
    vec3 normal;
    vec2 tex_coord;
    vec3 world_pos;
    mat3 tbn;
} vs_out;

void main() {

    vec4 position = vec4(i_position, 1.0);

    gl_Position = u_view_proj * u_model * position;

    vs_out.normal = i_normal;
    vs_out.tex_coord = i_tex_coord;
    vs_out.world_pos = (u_model * position).xyz; 

    vec3 T = normalize( vec3(u_model * vec4(i_tangent.xyz, 0.0)) );
    vec3 N = normalize( vec3(u_model * vec4(i_normal, 0.0)) );

    T = normalize(T - dot(T, N) * N);

    vec3 B = cross(N, T);

    vs_out.tbn = mat3(T, B, N);
}