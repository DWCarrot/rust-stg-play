#version 330

in vec3 position;
in vec2 txcoord;
in vec3 norm;
in vec3 offset;

out vec2 outTexCoord;

void main() {
    outTexCoord = txcoord;
    vec3 ang = norm;
    ang.x *= sign(float(gl_InstanceID % 3 == 0) - 0.5);
    ang.y *= sign(float(gl_InstanceID % 7 == 0) - 0.5);
    ang.z *= sign(float(gl_InstanceID % 13 == 0) - 0.5);
    vec3 c = cos(ang);
    vec3 s = sin(ang);
    mat3 t1 = mat3(
        c.x, -s.x,  0.0,
        s.x,  c.x,  0.0,
        0.0,  0.0,  1.0
    );
    mat3 t2 = mat3(
        c.y,  0.0, -s.y,      
        0.0,  1.0,  0.0,
        s.y,  0.0,  c.y
    );
    mat3 t3 = mat3(
        1.0,  0.0,  0.0,
        0.0,  c.z, -s.z,
        0.0,  s.z,  c.z
    );
    mat3 t0 = mat3(
        0.05, 0.0, 0.0,
        0.0, 0.05, 0.0,
        0.0, 0.0, 0.05
    );
    gl_Position = vec4(t0 * t1 * t2 * t3 * position + offset, 1.0);
}