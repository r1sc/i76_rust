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

const bool useTexture = true;
const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec3 reverseLightDir = -normalize(vec3(0.0, -1.0, -1.0));
const vec3 ambientColor = vec3(0.3, 0.3, 0.3);
const vec3 specularColor = vec3(1.0, 1.0, 1.0);
const float specularShininess = 64.0;
const float screenGamma = 1.0; // Assume the monitor is calibrated to the sRGB color space

void main() {
    // ambient
    vec3 ambient = lightColor * ambientColor;
  	
    // diffuse 
    vec3 norm = normalize(v_normal);
    float diff = max(dot(norm, reverseLightDir), 0.3);
    vec3 diffuse = lightColor * diff * v_color;
    
    if(useTexture) {
        diffuse *= texture(u_texture, v_uv).rgb;
    }
    
    // specular
    vec3 viewDir = normalize(-v_vertpos);
    vec3 reflectDir = reflect(reverseLightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), specularShininess);
    vec3 specular = lightColor * (spec * specularColor);
        
    vec3 colorLinear = ambient + diffuse + specular;

    // apply gamma correction (assume ambientColor, diffuseColor and specColor
    // have been linearized, i.e. have no gamma correction in them)
    vec3 colorGammaCorrected = pow(colorLinear, vec3(1.0 / screenGamma));

    color = vec4(colorGammaCorrected, 1.0);
}