#version 450

layout(push_constant) uniform PushConstantData {
	int time; 
	int dtime; 
} pc;

layout(points) in; 

layout(triangle_strip, max_vertices = 32) out; 
layout(location = 0) out vec2 uv; 

void transform_and_emit(in vec3 pos) {
	gl_Position = vec4(pos / pos.z, 1.0);
	EmitVertex();
}

void main() {
	float fish_size = 0.1; 
	vec3 head = 	vec3(1.0, 0.0, 0.0) * fish_size; 
	vec3 bottom = 	vec3(0.0, 1.0, 0.0) * fish_size;
	vec3 waves = 	vec3(0.0, 0.0, 0.08) * fish_size;
	
	int fragments = 16; // todo: determine based on distance to eye. MAX = max_vertices / 2

	vec3 pos; 
	float x;
	for(int i = 0; i < fragments; i++) {
		x = i * 1.0 / (fragments - 1);
		uv = vec2(x, 0);
		pos = gl_in[0].gl_Position.xyz
			+ x * head 
			+ pow(1 - x, 1.5) * sin(180.0 * x + pc.time * 0.005) * waves; 
		transform_and_emit(pos);
		
		uv = vec2(x, 1);
		pos += bottom;
		transform_and_emit(pos); 
	}
}
