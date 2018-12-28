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
    //fcolor = vec4((p.x + 1.0)/2, (p.y + 1.0)/2, p.z, 0.5 + p.y/2);
    fcolor = vec4((p.x + 1.0)/2, (p.y + 1.0)/2, p.z, 1.0);
}
