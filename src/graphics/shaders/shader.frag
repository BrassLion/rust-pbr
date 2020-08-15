#version 450

layout(location = 0) out vec4 f_color;

layout(location = 0) in VS_OUT
{
    vec3 normal;
} vs_in;

void main() {
    f_color = vec4(vs_in.normal, 0.5);
}