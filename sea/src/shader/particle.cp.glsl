#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in; 

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
	float friction_95;
} pc;

struct Particle {
	vec3 position; 
	vec3 offset;
	vec3 drift; 
};

struct Vertex {
	vec4 position; 
	vec4 tail; 
};

layout(set = 0, binding = 0) uniform sampler3D flux; 

layout(set = 0, binding = 1) buffer Particles {
	Particle particles[];
}; 

layout(set = 0, binding = 2) buffer writeonly Vertices {
	Vertex vertices[];
}; 

const float acceleration = 0.1; 
const float drift_factor = 0.2;
const float noisyness = 0.1; // later based on species

void main() {
	uint id = gl_GlobalInvocationID.x; 
	vec3 stream = texture(flux, particles[id].position * 0.5 + 0.5).rgb;
	stream.g *= 0.25; //fish don't move so much along this axis

	/* particles[id].velocity *= pc.friction_95;
	float o_distance = length(particles[id].position);
	float attraction; 
	if(o_distance < 0.1) { //this threshold increases temporarily, when vr head is rotated
		attraction = -0.3;
	} else {
		attraction = noisyness * o_distance;
	} 	
	particles[id].velocity += (
		stream * acceleration
	  + particles[id].drift * drift_factor 
	  - particles[id].position / o_distance * attraction 
	  
	) * pc.dtime;
	particles[id].position += (particles[id].velocity) * pc.dtime; */

	vec3 v = stream
		+ particles[id].drift * drift_factor
		- particles[id].position * noisyness;

	particles[id].position += v * pc.dtime; 
	vertices[id].position.rgb = particles[id].position.rgb + particles[id].offset * 0.1; 
	vertices[id].position.a = 1.0;

	float l = length(stream);
	if(l > 0) {
		vertices[id].tail = vec4(-stream, l);
	}
}

