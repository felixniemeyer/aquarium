#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(points) in; 

layout(triangle_strip, max_vertices = 32) out; 
layout(location = 0) out vec2 uv; 
layout(location = 1) out vec4 lighting_gs_out; 

const float undulation = 0.05;
const float fish_size = 0.1; 

layout(location = 0) in vec3 rear[]; 
layout(location = 1) in vec3 down[]; 
layout(location = 2) in vec3 side[]; 
layout(location = 3) in vec4 lighting[]; 

void transform_and_emit(in vec3 v) {
	vec4 position = gl_in[0].gl_Position;

	vec3 outp = position.xyz + v * position.a; 
	gl_Position = pc.viewPerspective * vec4(outp, 1);

	EmitVertex();
}

void emitTwo(in float p, in float x, in float z) {
	lighting_gs_out = lighting[0];
	uv = vec2(1 - p, 0); // head on the texture is right, we want it left so it fits the undulation
	vec3 offset = x * rear[0] + z * side[0];

	transform_and_emit(offset - 0.1 * down[0]);
	
	uv = vec2(1 - p, 1);
	transform_and_emit(offset + 0.9 * down[0]); 
}

void main() {
	int fragments = 16 - 1; // todo: determine based on distance to eye. MAX = max_vertices / 2 - 1

	float p;
	float x = -0.5; 
	float z, prev_z = 0.0; 
	float dz;  
	float c_square = pow(1.0 / fragments, 2); 
	
	emitTwo(0, x, prev_z);

	for(int i = 0; i < fragments; i++) {
		p = (i + 1.0) / fragments;

		// calculate z based on sin, then choose x so that the fish length does not get distorted
		z = pow(p, 1.5) * sin(180.0 * p + pc.time * 5 + gl_PrimitiveIDIn) * undulation;
		dz = z - prev_z; 
		x += sqrt(c_square - dz * dz); 

		prev_z = z;
	
		emitTwo(p, x, z);
	}
}
