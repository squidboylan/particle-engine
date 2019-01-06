#version 330 core
layout (location = 0) in vec4 vert_position;
layout (location = 1) in vec4 offset;
layout (location = 2) in vec4 color;
layout (location = 4) in float size;

out vec4 fcolor;

void main()
{
    gl_Position = vec4((vert_position.xyz * size) + offset.xyz, 1.0);
    fcolor = color;
}
