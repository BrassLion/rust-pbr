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
    vec3 normal;
    vec2 tex_coord;
    vec3 world_pos;
    mat3 tbn;
} vs_out;

void main() {

    vec4 position = vec4(i_position, 1.0);
    vec4 normal = vec4(i_normal, 1.0);

    gl_Position = u_camera.proj * u_camera.view * u_camera.model * position;

    vs_out.normal = (u_camera.model * normal).xyz;
    vs_out.tex_coord = i_tex_coord;
    vs_out.world_pos = (u_camera.model * position).xyz;

    vec3 T = normalize( (u_camera.model * vec4(i_tangent.xyz, 0.0)).xyz );
    vec3 N = normalize( vs_out.normal );
    vec3 B = normalize( (u_camera.model * vec4( (cross(i_tangent.xyz, i_normal) * i_tangent.w), 0.0 )).xyz );
    
    vs_out.tbn = mat3(T, B, N);
}