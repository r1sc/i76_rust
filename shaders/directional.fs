#version 330 core
precision highp float;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;
in vec3 v_fragPosition;

uniform mat4 u_model;
uniform vec3 u_ambient;
uniform sampler2D u_texture;

out vec4 color;

const vec3 lightColor = vec3(0.0, 0.5, 0.5);
const vec3 lightDir = -normalize(vec3(0.0, 0.0, -1.0));
const vec3 ambientColor = vec3(0.3, 0.3, 0.3);

void main() {
    float brightness = clamp(dot(v_normal, lightDir), 0.3, 1.0);

    vec3 diffuse = brightness * texture(u_texture, v_uv).xyz;
    color = vec4(diffuse + ambientColor, 1.0);
    // color = vec4(v_normal, 1.0);
}