#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in; 

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
} pc;

struct Particle {
	vec4 position; 
};

struct Vertex {
	vec4 position; 
};

layout(set = 0, binding = 0) uniform sampler3D flux; 

layout(set = 0, binding = 1) buffer Particles {
	Particle particles[];
}; 

layout(set = 0, binding = 2) buffer writeonly Vertices {
	Vertex vertices[];
}; 

const float speed = 0.1; 

void main() {
	uint id = gl_GlobalInvocationID.x; 
	vec3 stream = texture(flux, particles[id].position.rgb * 0.5 + 0.5).rgb;

	particles[id].position.rgb += stream * pc.dtime * speed;
	vertices[id].position.rgb = particles[id].position.rgb;

	// write tail = normalize(-stream) to vertices

}

