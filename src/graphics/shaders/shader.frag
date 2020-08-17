#version 450

layout(location = 0) out vec4 f_color;

layout(set=0, binding=0)
uniform Transforms {
    mat4 u_model;
    mat4 u_view_proj;
    vec3 u_camera_world_position;
};

layout(set=1, binding=0)
uniform Light {
    vec3 position;
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
} vs_in;

void main() {

    // Load material parameters.
    vec3 albedo     = pow(texture(sampler2D(t_albedo, s_albedo), vs_in.tex_coord).rgb, vec3(2.2));
    vec3 normal     = texture(sampler2D(t_normal, s_normal), vs_in.tex_coord).rgb;
    float metallic  = texture(sampler2D(t_metal_roughness, s_metal_roughness), vs_in.tex_coord).b;
    float roughness = texture(sampler2D(t_metal_roughness, s_metal_roughness), vs_in.tex_coord).g;
    float ao        = texture(sampler2D(t_ao, s_ao), vs_in.tex_coord).g;
    vec3 emissive   = texture(sampler2D(t_emissive, s_emissive), vs_in.tex_coord).rgb;

    // Convert normal from tangent space to world space.
    normal = normalize(normal * 2.0 - 1.0);

    // Phong shading.
    vec3 norm = normalize(vs_in.normal);
    vec3 light_direction = normalize(u_light.position - vs_in.world_pos);
    vec3 view_direction = normalize(u_camera_world_position - vs_in.world_pos);
    vec3 reflect_direction = reflect(-light_direction, norm);  
    float spec = pow(max(dot(view_direction, reflect_direction), 0.0), 32.0);

    vec3 ambient = emissive;
    float diffuse = max( dot(norm, light_direction), 0.0 );
    vec3 specular = vec3(1.0 * spec); 

    vec3 result = (ambient + diffuse + specular) * albedo;

    f_color = vec4(result, 1.0);
}