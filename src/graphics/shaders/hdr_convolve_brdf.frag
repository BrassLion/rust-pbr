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

layout(location = 0) out vec2 f_colour;


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

float geometry_schlick_ggx(float n_dot_v, float roughness)
{
    float r = roughness;
    float k = (r * r) / 2.0;

    float nom   = n_dot_v;
    float denom = n_dot_v * (1.0 - k) + k;
	
    return nom / denom;
}

float geometry_smith(vec3 normal, vec3 view_dir, vec3 light_dir, float roughness)
{
    float n_dot_v = max(dot(normal, view_dir), 0.0);
    float n_dot_l = max(dot(normal, light_dir), 0.0);
    float ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    float ggx2 = geometry_schlick_ggx(n_dot_l, roughness);
	
    return ggx1 * ggx2;
}

vec2 integrate_brdf(float n_dot_v, float roughness)
{
    vec3 V;
    V.x = sqrt(1.0 - n_dot_v * n_dot_v);
    V.y = 0.0;
    V.z = n_dot_v;

    float A = 0.0;
    float B = 0.0;

    vec3 N = vec3(0.0, 0.0, 1.0);

    const uint SAMPLE_COUNT = 1024u;
    for(uint i = 0u; i < SAMPLE_COUNT; ++i)
    {
        vec2 Xi = hammersley(i, SAMPLE_COUNT);
        vec3 H  = importance_sample_ggx(Xi, N, roughness);
        vec3 L  = normalize(2.0 * dot(V, H) * H - V);

        float n_dot_l = max(L.z, 0.0);
        float n_dot_h = max(H.z, 0.0);
        float v_dot_h = max(dot(V, H), 0.0);

        if(n_dot_l > 0.0)
        {
            float g = geometry_smith(N, V, L, roughness);
            float g_vis = (g * v_dot_h) / (n_dot_h * n_dot_v);
            float f_c = pow(1.0 - v_dot_h, 5.0);

            A += (1.0 - f_c) * g_vis;
            B += f_c * g_vis;
        }
    }

    A /= float(SAMPLE_COUNT);
    B /= float(SAMPLE_COUNT);

    return vec2(A, B);
}

void main() 
{
    f_colour = integrate_brdf(fs_in.tex_coord.x, fs_in.tex_coord.y);
}