#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(location = 0) in vec4 position; 
layout(location = 1) in vec4 tail; 

layout(location = 0) out mat4 rotation; 
layout(location = 4) out vec4 look_from; 

const vec3 UP = vec3(0, -1, 0);
const float FOG_AMOUNT = 0.5;

void main() { 
	gl_Position = position;  

	vec3 view_direction = position.xyz - pc.cameraPos;
	float cam_distance = length(view_direction); 
	look_from.rgb = view_direction / cam_distance; // normalize
	look_from.a = 1.0 / (1.0 + FOG_AMOUNT * cam_distance);

	/////
	vec3 rear = tail.xyz / tail.w; 
	vec3 side = normalize(cross(
		rear,
		UP
	));
	vec3 down = normalize(cross(
		rear, 
		side
	));

	rotation = mat4(
		vec4(rear, 0),
		vec4(down, 0),
		vec4(side, 0),
		vec4(0, 0, 0, 1)
	); 
}

