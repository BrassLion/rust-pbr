#version 450

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_ambient;
layout(set = 0, binding = 1) uniform sampler s_ambient;

layout(location = 0) in VS_OUT
{
    vec3 normal;
    vec2 tex_coord;
} vs_in;

void main() {

    f_color = texture(sampler2D(t_ambient, s_ambient), vs_in.tex_coord);
    // f_color = vec4(vs_in.tex_coord, 0.0, 1.0);
}