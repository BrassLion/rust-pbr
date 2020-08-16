#version 450

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_ambient;
layout(set = 0, binding = 1) uniform sampler s_ambient;

layout(set=1, binding=0)
uniform Light {
    vec3 position;
} u_light;

layout(location = 0) in VS_IN
{
    vec3 normal;
    vec2 tex_coord;
    vec3 world_pos;
} vs_in;

void main() {

    vec3 norm = normalize(vs_in.normal);
    vec3 light_direction = normalize(u_light.position - vs_in.world_pos);

    float diffuse = max( dot(norm, light_direction), 0.0 );

    f_color = diffuse * texture(sampler2D(t_ambient, s_ambient), vs_in.tex_coord);
    // f_color = diffuse * vec4(vs_in.tex_coord, 0.0, 1.0);
}