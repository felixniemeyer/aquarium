#version 450

layout(location = 0) in vec2 tex_coords; 

layout(location = 0) out vec4 f_color; 

// layout(set = 0, binding = 0) uniform sampler2D tex; 

void main() {
 	f_color = vec4(tex_coords, 0.5, 1.0); 
}
