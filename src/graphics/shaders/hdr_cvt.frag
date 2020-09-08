#version 450

layout(location = 0)
in VS_OUT {
    vec2 tex_coord;
    vec3 pos;
} fs_in;

layout(set = 1, binding = 0) uniform texture2D t_equirectangular;
layout(set = 1, binding = 1) uniform sampler s_equirectangular;

layout(location = 0) out vec4 f_colour;


const vec2 inv_atan = vec2(0.1591, 0.3183);
vec2 sample_spherical_map(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= inv_atan;
    uv += 0.5;
    return uv;
}

void main()
{		
    vec2 uv = sample_spherical_map(normalize(fs_in.pos));
    vec3 colour = texture(sampler2D(t_equirectangular, s_equirectangular), uv).rgb;
    
    f_colour = vec4(colour, 1.0);
}