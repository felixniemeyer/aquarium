#version 450

layout(location = 0) in vec2 tex_coords; 

layout(location = 0) out vec4 f_color; 

layout(set = 0, binding = 0) uniform sampler3D flux; 

void main() {
 	f_color = vec4(0.5 * (tex_coords + texture(flux, vec3(tex_coords, 0.5)).rg), 0.5, 1.0); 
}
