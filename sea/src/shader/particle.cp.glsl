#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in; 

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
	float friction_95;
} pc;

struct Particle {
	vec4 position; 
	vec4 offset;
	vec4 drift; 
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

const float speed = 1.0; 
const float drift_factor = 0.15; // a bigger drift factor leads to more individual fish paths
const float noisyness = 0.16; // later based on species

void main() {
	uint id = gl_GlobalInvocationID.x; 

	vec3 stream = texture(flux, particles[id].position.xyz * 0.5 + 0.5).rgb;
	stream.g *= 0.5; //fish don't move so much along this axis

	vec3 v = speed * (stream
		+ particles[id].drift.xyz * drift_factor
		- particles[id].position.xyz * noisyness
	);

	particles[id].position.xyz += v * pc.dtime; 
	vertices[id].position.xyz = particles[id].position.xyz + particles[id].offset.xyz * 0.1; 
	vertices[id].position.a = particles[id].position.a;

	v.y *= 0.5;
	float l = length(v);
	if(l > 0) {
		vertices[id].tail = vec4(-v, l);
	}
}

