#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(location = 0) in vec2 uv; 
layout(location = 1) in vec4 lighting_gs_out; // include alpha according to cam distance

layout(location = 0) out vec4 f_color; 

layout(set = 0, binding = 0) uniform sampler2D fish_colors; 
layout(set = 0, binding = 1) uniform sampler2D fish_normals; 

void main() {
	vec4 t = texture(fish_normals, uv) * texture(fish_colors, uv);
	if(t.a < 0.5) {
		discard;
	} else {
		f_color = t * lighting_gs_out;
	}
}
