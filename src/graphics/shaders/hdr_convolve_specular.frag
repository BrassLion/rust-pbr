#version 450

const float PI = 3.14159265359;

layout(location = 0)
in VS_OUT {
    vec2 tex_coord;
    vec3 pos;
} fs_in;

layout(set = 1, binding = 0) uniform textureCube t_environmentMap;
layout(set = 1, binding = 1) uniform sampler s_environmentMap;

layout(set = 2, binding = 0) uniform Material
{
    float roughness;
} u_material;

layout(location = 0) out vec4 f_colour;


float radical_inverse_vdc(uint bits) 
{
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}

vec2 hammersley(uint i, uint N)
{
    return vec2(float(i) / float(N), radical_inverse_vdc(i));
}

vec3 importance_sample_ggx(vec2 Xi, vec3 N, float roughness)
{
    float a = roughness * roughness;
	
    float phi = 2.0 * PI * Xi.x;
    float cos_theta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
	
    // Convert spherical to cartesian coordinates
    vec3 H;
    H.x = cos(phi) * sin_theta;
    H.y = sin(phi) * sin_theta;
    H.z = cos_theta;
	
    // Convert tangent-space vector to world-space vector
    vec3 up        = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent   = normalize(cross(up, N));
    vec3 bitangent = cross(N, tangent);
	
    vec3 result = tangent * H.x + bitangent * H.y + N * H.z;

    return normalize(result);
}  

void main()
{		
    vec3 N = normalize(fs_in.pos);  
    N.y = -N.y;  
    vec3 R = N;
    vec3 V = R;

    const uint SAMPLE_COUNT = 1024u;
    float total_weight = 0.0;   
    vec3 prefiltered_colour = vec3(0.0); 

    for(uint i = 0u; i < SAMPLE_COUNT; ++i)
    {
        vec2 x_i = hammersley(i, SAMPLE_COUNT);
        vec3 H  = importance_sample_ggx(x_i, N, u_material.roughness);
        vec3 L  = normalize(2.0 * dot(V, H) * H - V);

        float n_dot_l = max(dot(N, L), 0.0);

        if(n_dot_l > 0.0)
        {
            prefiltered_colour += texture(samplerCube(t_environmentMap, s_environmentMap), L).rgb * n_dot_l;
            total_weight       += n_dot_l;
        }
    }

    prefiltered_colour = prefiltered_colour / total_weight;

    f_colour = vec4(prefiltered_colour, 1.0);
} 