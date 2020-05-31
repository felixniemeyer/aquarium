#version 450

layout(local_size_x = 32, local_size_y = 1, local_size_z = 1) in; 

struct Particle {
	vec2 pos; 
	vec2 tail; 
	vec2 speed; 
	vec2 prev_pos; 
	vec2 prev_tail; 
};

layout(set = 0, binding = 0) buffer Particles {
	Particle particle[];
} particles; 

void main() {
	uint idx = gl_GlobalInvocationID.x; 
	float dtime = 1.0 / 60.0; //provide as uniform later on 
	Particle p = particles.particle[idx]; 
	p.prev_pos = p.pos; 
	p.prev_tail = p.tail; 
	p.speed += p.speed; // + gravity * dtime
	p.pos += p.speed * dtime; 
	p.tail = p.pos + vec2(0.0, 0.1); // based on gravity map
}

