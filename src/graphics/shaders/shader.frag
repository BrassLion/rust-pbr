#version 450

#define PI 3.1415926538

layout(location = 0) out vec4 f_colour;

layout(set=0, binding=0)
uniform Transforms {
    mat4 model;
    mat4 view_proj;
    vec3 world_pos;
} u_camera;

layout(set=1, binding=0)
uniform Light {
    vec3 world_pos;
} u_light;

layout(set = 2, binding = 0) uniform texture2D t_ao;
layout(set = 2, binding = 1) uniform sampler s_ao;
layout(set = 2, binding = 2) uniform texture2D t_albedo;
layout(set = 2, binding = 3) uniform sampler s_albedo;
layout(set = 2, binding = 4) uniform texture2D t_emissive;
layout(set = 2, binding = 5) uniform sampler s_emissive;
layout(set = 2, binding = 6) uniform texture2D t_metal_roughness;
layout(set = 2, binding = 7) uniform sampler s_metal_roughness;
layout(set = 2, binding = 8) uniform texture2D t_normal;
layout(set = 2, binding = 9) uniform sampler s_normal;

layout(location = 0)
in VS_IN {
    vec3 normal;
    vec2 tex_coord;
    vec3 world_pos;
    mat3 tbn;
} vs_in;

float distribution_ggx(vec3 normal, vec3 half_dir, float roughness)
{
    float a         = roughness * roughness;
    float a_2       = a * a;
    float n_dot_h   = max(dot(normal, half_dir), 0.0);
    float n_dot_h_2 = n_dot_h * n_dot_h;
	
    float nom    = a_2;
    float denom  = (n_dot_h_2 * (a_2 - 1.0) + 1.0);
    denom        = PI * denom * denom;
	
    return nom / denom;
}

float geometry_schlick_ggx(float n_dot_v, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

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

vec3 fresnel_schlick(float cos_theta, vec3 fresnel_0)
{
    return fresnel_0 + (1.0 - fresnel_0) * pow(1.0 - cos_theta, 5.0);
}

void main() {

    // Load material parameters.
    vec3 albedo     = pow(texture(sampler2D(t_albedo, s_albedo), vs_in.tex_coord).rgb, vec3(2.2));
    vec3 normal     = texture(sampler2D(t_normal, s_normal), vs_in.tex_coord).rgb;
    float metallic  = texture(sampler2D(t_metal_roughness, s_metal_roughness), vs_in.tex_coord).b;
    float roughness = texture(sampler2D(t_metal_roughness, s_metal_roughness), vs_in.tex_coord).g;
    float ao        = texture(sampler2D(t_ao, s_ao), vs_in.tex_coord).g;
    vec3 emissive   = texture(sampler2D(t_emissive, s_emissive), vs_in.tex_coord).rgb;

    // Convert normal from tangent space to world space.
    normal = normal * 2.0 - 1.0;
    normal = normalize(vs_in.tbn * normal);

    // Phong shading.
    // vec3 norm = normal;
    // vec3 light_direction = normalize(u_light.position - vs_in.world_pos);
    // vec3 view_direction = normalize(u_camera_world_position - vs_in.world_pos);
    // vec3 reflect_direction = reflect(-light_direction, norm);  
    // float spec = pow(max(dot(view_direction, reflect_direction), 0.0), 128.0);

    // vec3 ambient = emissive + vec3(0.1);
    // float diffuse = max( dot(norm, light_direction), 0.0 );
    // vec3 specular = vec3(3.0 * spec); 

    // PBR shading.

    // Calculate light properties.
    vec3 light_colour = vec3(23.47, 21.31, 20.79);

    vec3 light_dir = normalize(u_light.world_pos - vs_in.world_pos);
    vec3 view_dir = normalize(u_camera.world_pos - vs_in.world_pos);
    vec3 half_dir = normalize(view_dir + light_dir);

    float light_distance = length(u_light.world_pos - vs_in.world_pos);
    float light_attenuation = 1.0 / (light_distance * light_distance);
    vec3 light_radiance = light_colour * light_attenuation;

    // Calculate Cook-Torrance specular BRDF: DFG / 4(ωo⋅n)(ωi⋅n)
    vec3 fresnel_0 = mix(vec3(0.04), albedo, metallic);

    vec3 F = fresnel_schlick( max( dot(half_dir, view_dir), 0.0 ), fresnel_0 );
    float D = distribution_ggx(normal, half_dir, roughness);
    float G = geometry_smith(normal, view_dir, light_dir, roughness);

    float denom = 4.0 * max(dot(normal, view_dir), 0.0) * max(dot(normal, light_dir), 0.0);

    vec3 specular = (D * F * G) / max(denom, 0.001);

    // Calculate ratio of reflected-refracted light.
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;

    kD *= 1.0 - metallic;	

    // Calculate output radiance.
    float n_dot_l = max(dot(normal, light_dir), 0.0);

    vec3 L_0 = (kD * albedo / PI + specular) * light_radiance * n_dot_l;

    // Calculate final fragment colour.
    vec3 ambient = vec3(0.01) * albedo * ao + emissive;

    vec3 colour = ambient + L_0;

    // Gamma correct.
    colour = colour / (colour + vec3(1.0));
    colour = pow(colour, vec3(1.0 / 2.2));

    f_colour = vec4(colour, 1.0); 

    // f_colour = vec4(texture(sampler2D(t_metal_roughness, s_metal_roughness), vs_in.tex_coord).rgb, 1.0);
    // f_colour = vec4((normal + 1.0) / 2, 1.0);
}