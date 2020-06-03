#version 450

layout(push_constant) uniform PushConstantData {
	float time; 
	float dtime; 
} pc;

layout(points) in; 

layout(triangle_strip, max_vertices = 32) out; 
layout(location = 0) out vec2 uv; 

void transform_and_emit(in vec3 pos) {
	vec3 xyz = gl_in[0].gl_Position.xyz + pos; 
	gl_Position = vec4(xyz, xyz.z); 
	EmitVertex();
}

void main() {
	float fish_size = 0.1; 
	float length = 1.0 * fish_size; 
	float height = 1.0 * fish_size;
	float waves = 0.05 * fish_size;
	
	int fragments = 16 - 1; // todo: determine based on distance to eye. MAX = max_vertices / 2 - 1

	vec3 pos; 
	float p;
	float x = 0; 
	float z, prev_z = 0; 
	float dx, dz;  
	float c_square = pow(length / fragments, 2); 
	for(int i = 0; i <= fragments; i++) {
		p = i * 1.0 / fragments;

		// calculate z based on sin, then choose x so that the fish length does not get distorted
		z = pow(p, 1.5) * sin(180.0 * p + pc.time * 0.005 + gl_PrimitiveIDIn) * waves;
		dz = z - prev_z; 
		dx = sqrt(c_square - dz * dz); 

		x += dx; 
		prev_z = z;
		
		uv = vec2(1 - p, 0);
		transform_and_emit(vec3(x,0,z));
		
		uv = vec2(1 - p, 1);
		transform_and_emit(vec3(x,height,z)); 
	}
}
