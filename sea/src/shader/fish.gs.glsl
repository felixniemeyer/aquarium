#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(points) in; 
layout(location = 0) in mat4 rotation[]; 
layout(location = 4) in vec4 look_from[]; 

layout(triangle_strip, max_vertices = 32) out; 
layout(location = 0) out vec2 uv; 
layout(location = 1) out vec4 look_from_gs_out; 
layout(location = 2) out mat4 rotation_gs_out; 

const float amplitude = 0.055;


void transform_and_emit(in vec4 v) {
	vec4 position = gl_in[0].gl_Position;

	vec4 outp = vec4(position.xyz, 1) + position.a * rotation[0] * v ; 
	gl_Position = pc.viewPerspective * outp;

	EmitVertex();
}

void emitTwo(in float p, in float x, in float z) {
	rotation_gs_out = rotation[0];
	look_from_gs_out = look_from[0];

	uv = vec2(1 - p, 0); // head on the texture is right, we want it left so it fits the amplitude
	transform_and_emit(vec4(x, -0.5, z, 0)); //offset - 0.5 * down[0]);
	
	uv = vec2(1 - p, 1);
	transform_and_emit(vec4(x, 0.5, z, 0)); //offset + 0.5 * down[0]); 
}

void main() {
	int fragments = int(1 + pow(look_from[0].a, 2)  * (16 - 2)); // MAX = max_vertices / 2 - 1

	float p;
	float x = -0.5; 
	float z, prev_z = 0.0; 
	float dz;  
	float c_square = pow(1.0 / fragments, 2); 
	
	emitTwo(0, x, prev_z);

	float frequency = 8.5; 

	for(int i = 0; i < fragments; i++) {
		p = (i + 1.0) / fragments;

		// calculate z based on sin, then choose x so that the fish length does not get distorted
		z = (0.25 + p * 0.75) * sin(frequency * pow(1-p,1.5) + pc.time * 5 + gl_PrimitiveIDIn) * amplitude;
		dz = z - prev_z; 
		x += sqrt(c_square - dz * dz); 

		prev_z = z;
	
		emitTwo(p, x, z);
	}
}
