#version 330 core
in vec3 position;
in vec3 center;
in vec3 col;
in float size;

vec4 p;

out vec4 fcolor;

void main()
{
    p = vec4(position * size + center, 1.0);
    gl_Position = p;
    fcolor = vec4(col, 1.0);
}
