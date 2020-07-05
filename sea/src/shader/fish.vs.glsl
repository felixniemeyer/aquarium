#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(location = 0) in vec4 position; 
layout(location = 1) in vec4 tail; 

layout(location = 0) out vec3 rear; 
layout(location = 1) out vec3 down; 
layout(location = 2) out vec3 side; 
layout(location = 3) out vec4 lighting; 

const vec3 UP = vec3(0, -1, 0);
const float FOG_AMOUNT = 1.0;

void main() { 
	gl_Position = position;  

	vec3 view_direction = position.xyz - pc.cameraPos;
	float cam_distance = length(view_direction); 
	view_direction /= cam_distance; // normalize
	
	lighting.rgb = vec3(0.3,0.4,0.5)
		 + vec3(0.3,0.2,0.1) * pow(dot(vec3(0, 1, 0), view_direction) * 0.5 + 0.5, 2);

	lighting.a = 1.0 / (1.0 + FOG_AMOUNT * cam_distance);

	rear = tail.xyz / tail.w; 
	side = normalize(cross(
		rear,
		UP
	));
	down = normalize(cross(
		rear, 
		side
	));
}

