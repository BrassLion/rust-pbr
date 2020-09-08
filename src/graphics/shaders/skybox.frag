#version 450

layout(location = 0) out vec4 f_colour;

layout(set = 1, binding = 0) uniform textureCube t_environmentMap;
layout(set = 1, binding = 1) uniform sampler s_environmentMap;

layout(location = 0)
in VS_OUT {
    vec3 local_pos;
} vs_out;

void main()
{
    vec3 envColor = textureLod(samplerCube(t_environmentMap, s_environmentMap), vs_out.local_pos, 1.2).rgb;
    
    envColor = envColor / (envColor + vec3(1.0));
    envColor = pow(envColor, vec3(1.0/2.2)); 
  
    f_colour = vec4(envColor, 1.0);
}