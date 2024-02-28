#version 330 core

precision highp float;

in vec3 v_vertpos;
in vec3 v_normal;

uniform mat4 u_projection;
uniform mat4 u_modelview;
uniform sampler2D u_texture;

out vec4 color;

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec3 reverseLightDir = -normalize(vec3(0.0, -1.0, -1.0));
const vec3 ambientColor = vec3(0.3, 0.3, 0.3);

void main() {
    // ambient
    vec3 ambient = lightColor * ambientColor;
  	
    // diffuse 
    vec3 norm = normalize(v_normal);
    float diff = max(dot(norm, reverseLightDir), 0.3);
    vec3 diffuse = lightColor * diff * v_color.rgb;
    
    vec4 tex = texture(u_texture, v_uv);
    diffuse *= tex.rgb;    

    vec3 colorLinear = ambient + diffuse;

    // apply gamma correction (assume ambientColor, diffuseColor and specColor
    // have been linearized, i.e. have no gamma correction in them)
    vec3 colorGammaCorrected = pow(colorLinear, vec3(1.0 / screenGamma));

    color = vec4(colorGammaCorrected, 1.0);
}