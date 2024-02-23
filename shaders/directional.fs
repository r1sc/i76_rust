#version 330 core
precision highp float;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;
in vec3 v_vertpos;

uniform mat4 u_projection;
uniform mat4 u_modelview;
uniform sampler2D u_texture;

out vec4 color;

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec3 lightDir = normalize(vec3(0.0, -1.0, -1.0));
const vec3 ambientColor = vec3(0.3, 0.3, 0.3);
const vec3 specularColor = vec3(1.0, 1.0, 1.0);
const float specularShininess = 16.0;
const float screenGamma = 1.0; // Assume the monitor is calibrated to the sRGB color space

void main() {
    vec3 normal = normalize(v_normal);

    float lambertian = clamp(dot(-lightDir, normal), 0.0, 1.0);
    vec3 viewDir = normalize(-v_vertpos);
    
    float specular = 0.0;
    if(specularShininess > 0.0) {
        vec3 halfDir = normalize(-lightDir + viewDir);
        float specAngle = max(dot(halfDir, v_normal), 0.0);
        specular = pow(specAngle, specularShininess);
    }

    vec3 diffuseColor = v_color * texture(u_texture, v_uv).xyz;

    vec3 colorLinear = ambientColor +
                       diffuseColor * lambertian * lightColor +
                       specularColor * specular * lightColor;

    // apply gamma correction (assume ambientColor, diffuseColor and specColor
    // have been linearized, i.e. have no gamma correction in them)
    vec3 colorGammaCorrected = pow(colorLinear, vec3(1.0 / screenGamma));

    color = vec4(colorGammaCorrected, 1.0);
}