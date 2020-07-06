#version 450

layout(push_constant) uniform PCData {
	vec3 straight; 
	float dummy; 
	vec3 right; 
	float dummy2; 
	vec3 bottom; 
} pc; 

layout(location = 0) in vec2 tex_coords; 

layout(location = 0) out vec4 f_color; 

struct Light {
	vec3 color; 
	vec3 normal;
};
Light sun = Light(
	vec3(0.9, 0.3, 0.0), 
	vec3(0, -1, 0)
);
Light sea = Light(
	vec3(0.0, 0.3, 0.5) * 0.4,
	vec3(0, -1, 0)
);
Light white = Light(
	vec3(1.0, 1.0, 1.0) * 0.4,
	normalize(vec3(-1, 0.5, 0))
);

const float FRUSTUM_HALF = 0.1; 
const float FRUSTUM_NCP = 0.01;

void main() {
	vec3 normal = normalize(	
		pc.right * FRUSTUM_HALF * ( tex_coords.x * 2 - 1) * FRUSTUM_HALF
		+ pc.bottom * FRUSTUM_HALF * ( tex_coords.y * 2 - 1) * (-1) * FRUSTUM_HALF
		+ pc.straight * FRUSTUM_NCP
	);

	vec3 base_color = vec3(0.1, 0.2, 0.4) * 0.05;


	f_color = vec4(base_color 
		+ sun.color * pow(dot(sun.normal, normal)*0.5 + 0.5, 16)
		+ sea.color * pow(dot(sea.normal, normal)*0.5 + 0.5, 1) 
	, 1);

	// f_color = vec4(vec3(0.5) + 0.5 * normal, 1); 
	
}
