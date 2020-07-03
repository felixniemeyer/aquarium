#version 450

layout(location = 0) in vec4 position; 
layout(location = 1) in vec4 tail; 

layout(location = 0) out vec3 rear; 
layout(location = 1) out vec3 down; 
layout(location = 2) out vec3 side; 

const vec3 UP = vec3(0, -1, 0);

void main() { 
	gl_Position = position;  
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

