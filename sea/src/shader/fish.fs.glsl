#version 450

layout(push_constant) uniform PushConstantData {
	mat4 viewPerspective; 
	vec3 cameraPos; 
	float time; 
	float dtime; 
} pc;

layout(location = 0) in vec2 uv; 
layout(location = 1) in vec4 look_from; // include alpha according to cam distance
layout(location = 2) in mat4 rotation; 

layout(location = 0) out vec4 f_color; 

layout(set = 0, binding = 0) uniform sampler2D fish_colors; 
layout(set = 0, binding = 1) uniform sampler2D fish_normals; 

struct Light {
	vec3 color; 
	vec3 normal;
	float exponent; 
};
const Light sun = Light(
	vec3(0.9, 0.3, 0.0), 
	vec3(0, 1, 0),
	10.0
);
const Light sea = Light(
	vec3(0.0, 0.3, 0.5) * 0.4,
	vec3(0, 1, 0),
	1.0
);

void main() {
	vec4 c = texture(fish_colors, uv);
	if(c.a < 0.5) {
		discard;
	} else {
		vec4 normal = (texture(fish_normals, uv) - 0.5) * 2;
		normal.x = -normal.x; //well, for some reason...  coordinate systems... don't think just check... (nooot)
		if(!gl_FrontFacing) {
			normal.z = -normal.z;
		}
		vec3 n = (rotation * normal).xyz;
		vec4 light = vec4(
			- n * 0.4 + vec3(0.5)
			+ vec3(0.1)
			+ pow(dot(n, -sun.normal) * 0.5 + 0.5, sun.exponent) * sun.color * 2
			+ pow(dot(n, -sea.normal) * 0.5 + 0.5, sea.exponent) * sea.color
			 + pow(smoothstep(0.9,1.0,dot(normalize(-look_from.xyz + vec3(1,0.25,0)), n) * 0.5 + 0.5), 4.0) * 5* vec3(1,1,1) 
			// + pow(smoothstep(0.9,1.0,dot(normalize(look_from.xyz + sun.normal), n) * 0.5 + 0.5), 4.0) * sun.color * 3.0
			// + pow(smoothstep(0.7, 1.0, dot(n, -look_from.rgb) * 0.5 + 0.5), 4) * vec3(0.2)
		, look_from.a);

		f_color = c * light;
	}
}
