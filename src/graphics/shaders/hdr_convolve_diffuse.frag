#version 450

layout(location = 0) out vec4 f_colour;

layout(set = 1, binding = 0) uniform textureCube t_environmentMap;
layout(set = 1, binding = 1) uniform sampler s_environmentMap;

layout(location = 0) in vec3 f_pos;

const float PI = 3.14159265359;

void main()
{		
    vec3 normal = normalize(f_pos);

    vec3 irradiance = vec3(0.0);
    
    // Convolve the environment map.
    vec3 up    = vec3(0.0, 1.0, 0.0);
    vec3 right = cross(up, normal);
    up         = cross(normal, right);

    float sample_delta = 0.025;
    float nr_samples = 0.0; 
    for(float phi = 0.0; phi < 2.0 * PI; phi += sample_delta)
    {
        for(float theta = 0.0; theta < 0.5 * PI; theta += sample_delta)
        {
            // spherical to cartesian (in tangent space)
            vec3 tangent_sample = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            // tangent space to world
            vec3 sample_vec = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * normal; 

            irradiance += texture(samplerCube(t_environmentMap, s_environmentMap), sample_vec).rgb * cos(theta) * sin(theta);
            nr_samples++;
        }
    }
    irradiance = PI * irradiance * (1.0 / float(nr_samples));

    f_colour = vec4(irradiance, 1.0);
}