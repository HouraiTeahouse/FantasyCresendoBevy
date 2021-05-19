#version 450
layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj { mat4 ViewProj; };
layout(set = 1, binding = 0) uniform Capsule_start { vec3 Start; };
layout(set = 1, binding = 1) uniform Capsule_end { vec3 End; };
layout(set = 1, binding = 2) uniform Capsule_radius { float Radius; };

void main() {
    vec3 diff = End - Start;
    float side = sign(dot(Vertex_Position, diff));
    vec3 mid = 0.5 * diff + Start;
    vec3 position = (Vertex_Position * Radius) + mid;
    position += 0.5 * diff * side;
    gl_Position = ViewProj * vec4(position, 1.0);
}