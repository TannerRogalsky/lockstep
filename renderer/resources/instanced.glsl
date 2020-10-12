varying vec4 vColor;
varying vec2 vUV;
varying vec3 vPos;

#ifdef VERTEX
attribute vec4 position;
attribute vec3 normal;
attribute vec2 uv;

attribute vec4 color;
attribute vec4 offset;
attribute float scale;
attribute float angle;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;

mat2 rotate2d(float _angle){
    return mat2(cos(_angle),-sin(_angle),
                sin(_angle),cos(_angle));
}

void main() {
    vColor = color;
    vUV = uv;

    mat2 rotation = rotate2d(angle);
    vPos = position.xyz;
    // TODO: why is this doubling necessary?
    vec4 pos = uModel * vec4(vec3(2.), 1.) * (offset + position * mat4(rotation) * vec4(vec3(scale), 1.)) ;
    gl_Position = uProjection * uView * pos;
}
#endif

#ifdef FRAGMENT
uniform sampler2D tex0;
uniform vec4 uColor;

void main() {
    vec4 c = vec4(normalize(vPos), 1.0);
    fragColor = vColor * uColor * c;
}
#endif