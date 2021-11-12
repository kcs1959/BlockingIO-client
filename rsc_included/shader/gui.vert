#version 330 core

layout (location = 0) in vec3 iPosition;
layout (location = 1) in vec3 iNormal;  // not used
layout (location = 2) in vec2 iTexCoords;

uniform float uWidth;
uniform float uHeight;
uniform float uAlpha;

out float Alpha;
out vec3 FragPosition;
out vec3 Normal;
out vec2 TexCoords;

void main()
{
    Alpha = uAlpha;
    FragPosition = vec3(iPosition.x * 2 / uWidth, -iPosition.y * 2 / uHeight, iPosition.z) + vec3(-1.0, 1.0, 0.0);
    Normal = iNormal;
    TexCoords = iTexCoords;
    gl_Position = vec4(FragPosition, 1.0);
}
