#version 450

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
	float padding[2]; 
	mat4 viewPerspective; 
} pc;


layout(points) in; 

layout(triangle_strip, max_vertices = 32) out; 
layout(location = 0) out vec2 uv; 

const float undulation = 0.05;
const float fish_size = 0.1; 

layout(location = 0) in vec3 rear[]; 
layout(location = 1) in vec3 down[]; 
layout(location = 2) in vec3 side[]; 

void transform_and_emit(in vec3 v) {

	gl_Position = pc.viewPerspective *
		(gl_in[0].gl_Position + vec4(v * fish_size, 1)); 

	if(gl_Position.x == 0.0) {
		v = gl_in[0].gl_Position.xyz + v * fish_size + vec3(0,0,0.5); 
		gl_Position = vec4(v, v.z); 
	}

	EmitVertex();
}

void emitTwo(in float p, in float x, in float z) {
	uv = vec2(1 - p, 0); // head on the texture is right, we want it left so it fits the undulation
	vec3 offset = x * rear[0] + z * side[0];

	transform_and_emit(offset - 0.1 * down[0]);
	
	uv = vec2(1 - p, 1);
	transform_and_emit(offset + 0.9 * down[0]); 
}

void main() {
	int fragments = 16 - 1; // todo: determine based on distance to eye. MAX = max_vertices / 2 - 1

	vec3 offset; 
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
