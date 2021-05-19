#version 450
layout(location = 0) in vec3 v_Position;
layout(location = 1) in vec4 v_Color;

layout(location = 0) out vec4 o_Target;

void main() {
    if (v_Color.a < 0) {
        discard;
    }

    // Always render.
    gl_FragDepth = 0.0;
    o_Target = v_Color;
}