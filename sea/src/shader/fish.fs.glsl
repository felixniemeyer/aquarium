#version 450

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
	float padding[2]; 
	mat4 viewPerspective; 
} pc;

layout(location = 0) in vec2 uv; 

layout(location = 0) out vec4 f_color; 

layout(set = 0, binding = 0) uniform sampler2D fish_skin; 

void main() {
 	f_color = texture(fish_skin, uv); 
 	// f_color = vec4(0.1) + texture(fish_skin, uv) * 0.9; 
}
