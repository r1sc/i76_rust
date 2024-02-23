#version 330 core
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec3 a_normal;
layout(location = 3) in vec3 a_color;

out vec2 v_uv;
out vec3 v_normal;
out vec3 v_color;
out vec3 v_vertpos;

uniform mat4 u_projection;
uniform mat4 u_modelview;
uniform mat4 u_normal;

void main() {
    vec4 vertPos4 = u_modelview * vec4(a_position, 1.0);
    
    gl_Position = u_projection * vertPos4;
    
    v_vertpos = vec3(vertPos4) / vertPos4.w;
    v_uv = a_uv;
    v_normal = vec3(u_normal * vec4(a_normal, 0.0));
    v_color = a_color;
}