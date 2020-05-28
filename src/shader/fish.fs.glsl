#version 450

layout(location = 0) in vec2 uv; 

layout(location = 0) out vec4 f_color; 

layout(push_constant) uniform PushConstantData {
	int time; 
	int dtime; 
} pc;

layout(set = 0, binding = 0) uniform sampler2D fish_skin; 

void main() {
 	f_color = vec4(0.1) + texture(fish_skin, uv) * 0.9; 
}
