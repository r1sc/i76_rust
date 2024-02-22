#version 330 core
precision highp float;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;
in vec3 v_fragPosition_ViewSpace;

uniform mat4 u_view;
uniform mat4 u_model;
uniform vec3 u_ambient;
uniform sampler2D u_texture;

out vec4 color;

const vec3 lightColor = vec3(0.0, 0.5, 0.5);
const vec3 lightDir = -normalize(vec3(0.0, 0.0, -1.0));
const vec3 ambientColor = vec3(0.3, 0.3, 0.3);
const vec3 specularColor = vec3(1.0, 1.0, 1.0);
const float specularStrength = 0.5;
const float specularShininess = 32.0;

void main() {
    float NdotL = dot(v_normal, lightDir);
    float brightness = clamp(NdotL, 0.0, 1.0);
    vec3 diffuse = brightness * (v_color * texture(u_texture, v_uv).xyz);

    vec3 H = normalize(lightDir + normalize(v_fragPosition_ViewSpace));
    float NdotH = max(dot(v_normal, H), 0.0);
    float specularBrightness = pow(NdotH, 256.0) * specularStrength;
    vec3 specular = specularBrightness * specularColor;

    color = vec4(diffuse + specular + ambientColor, 1.0);
}