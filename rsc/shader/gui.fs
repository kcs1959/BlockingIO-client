#version 330 core

in float Alpha;
in vec3 FragPosition;
in vec3 Normal;
in vec2 TexCoords;

uniform sampler2D uScreenTexture;

out vec4 FragColor;

void main()
{
    vec4 texRGB = texture(uScreenTexture, TexCoords).rgba;
    FragColor = vec4(texRGB.rgb, texRGB.a * Alpha);
}