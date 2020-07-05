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
const float FOG_AMOUNT = 0.3;

struct Light {
	vec3 color; 
	vec3 normal;
	float e; 
};
const Light sun = Light(
	vec3(0.9, 0.3, 0.0), 
	vec3(0, -1, 0),
	2.0
);
const Light sea = Light(
	vec3(0.0, 0.3, 0.5) * 0.4,
	vec3(0, -1, 0),
	1.0
);

void main() { 
	gl_Position = position;  

	vec3 view_direction = position.xyz - pc.cameraPos;
	float cam_distance = length(view_direction); 
	view_direction /= cam_distance; // normalize
	
	lighting.rgb = vec3(0.3,0.3,0.3) 
		 + sun.color * pow(dot(-sun.normal, view_direction) * 0.5 + 0.5, sun.e);
		 + sea.color * pow(dot(-sea.normal, view_direction) * 0.5 + 0.5, sea.e);

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

