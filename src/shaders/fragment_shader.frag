#version 330 core

in vec2 texture_coord;
in vec3 frag_normal;
in vec3 frag_position;

out vec4 frag_colour;

struct LightingProperties {
    vec3 normal;
    vec3 frag_to_camera;
    vec3 view_dir;
};

struct GenericLight {
    vec3 light_dir;
    
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct GenericOutput {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
};

struct PointLight {
    vec3 position;
  
    vec3 attenuation;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct FarLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct SpotLight {
    vec3 position;
    vec3 direction;

    vec3 attenuation;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float cos_cut_off;
    float outer_cut_off;
    float cos_outer_cut_off;
};

struct Material {
    sampler2D diffuse; // background requires no mask
    sampler2D specular_map; // specular map is a mask 
    sampler2D emission; // emission is a texture and so needs a mask
    sampler2D emission_map;

    float shininess;
};

uniform Material material;
uniform PointLight light;
uniform FarLight sun;
uniform SpotLight torch;

uniform float time;
uniform vec3 camera_position;

LightingProperties lighting_properties();
GenericOutput generic_light(GenericLight, LightingProperties);
float attenuation(vec3);
vec4 PointLight_illuminate(PointLight, LightingProperties);
vec4 FarLight_illuminate(FarLight, LightingProperties);
vec4 SpotLight_illuminate(SpotLight, LightingProperties);

void main() {
    
    LightingProperties prop = lighting_properties();

    // Illumination
    vec4 illumination = 
        + PointLight_illuminate(light, prop)
        + FarLight_illuminate(sun, prop)
        + SpotLight_illuminate(torch, prop)
    ;
    
    // Emission
    vec4 emission = texture(material.emission, texture_coord + vec2(0, 0.5 * time))
        * texture(material.emission_map, texture_coord);
    
    frag_colour = illumination + emission;
    
    if (frag_colour.a < 0.01) {
        discard;
    }
}

LightingProperties lighting_properties() {
    vec3 normal = normalize(frag_normal);
    vec3 frag_to_camera = camera_position - frag_position;
    vec3 view_dir = normalize(frag_to_camera);

    return LightingProperties (
        normal,
        frag_to_camera,
        view_dir
    );
}

GenericOutput generic_light(GenericLight light, LightingProperties prop) {
    vec4 diffuse_map = texture(material.diffuse, texture_coord);
    
    // Ambient
    vec4 ambient = vec4(light.ambient, 1.0) * diffuse_map;
    
    // Diffuse
    float diffuse_intensity = max(0.0, dot(prop.normal, light.light_dir));
    vec4 diffuse = vec4(light.diffuse, 1.0) * diffuse_intensity * diffuse_map;

    // Specular
    vec4 specular_map = texture(material.specular_map, texture_coord);

    vec3 reflect_dir = reflect(-light.light_dir, prop.normal);
    float cos_reflect_angle = max(dot(prop.view_dir, reflect_dir), 0.0);

    vec4 specular = vec4(light.specular, 1.0)
        * pow(cos_reflect_angle, material.shininess)
        * specular_map;

    return GenericOutput (
        ambient,
        diffuse,
        specular
    );
}

vec4 attenuation(vec3 factors, float light_dist) {
    float x =  1.0 / (
        factors.x
        + (factors.y * light_dist)
        + (factors.z * light_dist * light_dist)
    );

    return vec4(vec3(x), 1.0);
}

vec4 PointLight_illuminate(PointLight light, LightingProperties prop) {
    vec3 frag_to_light = light.position - frag_position;

    GenericLight gen_light = GenericLight(
        normalize(frag_to_light),
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light, prop);

    // Attenuation
    float light_dist = length(frag_to_light);
    vec4 light_attenuation = attenuation(light.attenuation, light_dist);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation;
}

vec4 FarLight_illuminate(FarLight light, LightingProperties prop) {
    GenericLight gen_light = GenericLight(
        normalize(-light.direction),
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light, prop);

    return gen_out.ambient + gen_out.diffuse + gen_out.specular;
}

vec4 SpotLight_illuminate(SpotLight light, LightingProperties prop) {
    vec3 frag_to_light = light.position - frag_position;
    vec3 light_dir = normalize(frag_to_light);
    
    float theta     = dot(light_dir, normalize(-light.direction));
    float epsilon   = light.cos_cut_off - light.cos_outer_cut_off;
    float intensity = clamp((theta - light.cos_outer_cut_off) / epsilon, 0.0, 1.0);

    GenericLight gen_light = GenericLight(
        light_dir,
        light.ambient,
        light.diffuse,
        light.specular
    );

    GenericOutput gen_out = generic_light(gen_light, prop);

    // Attenuation
    float light_dist = length(frag_to_light);
    vec4 light_attenuation = attenuation(light.attenuation, light_dist);

    // Return
    return gen_out.ambient + (gen_out.diffuse + gen_out.specular) * light_attenuation * intensity;
}