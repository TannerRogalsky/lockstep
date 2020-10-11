varying vec4 vColor;
varying vec2 vUV;

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
    mat4 rotation = mat4(rotate2d(angle));
    // TODO: why is this doubling necessary?
    vec4 pos = uModel * vec4(vec3(2.), 1.) * (offset + position * rotation * vec4(vec3(scale), 1.)) ;
    gl_Position = uProjection * uView * pos;
}
#endif

#ifdef FRAGMENT
uniform sampler2D tex0;
uniform vec4 uColor;

void main() {
    fragColor = vColor * uColor * vec4(vUV.x, vUV.y, (vUV.x + vUV.y) / 2., 1.);
}
#endif