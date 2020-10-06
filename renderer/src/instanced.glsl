varying vec4 vColor;
varying vec2 vUV;

#ifdef VERTEX
attribute vec4 position;
attribute vec4 color;
attribute vec2 uv;

attribute vec4 offset;
attribute float scale;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;

void main() {
    vColor = color;
    vUV = uv;
    // TODO: why is this doubling necessary?
    vec4 pos = uModel * vec4(vec3(2.), 1.) * (offset + position * vec4(vec3(scale), 1.)) ;
    gl_Position = uProjection * uView * pos;
}
#endif

#ifdef FRAGMENT
uniform sampler2D tex0;
uniform vec4 uColor;

void main() {
    fragColor = vColor * uColor;
}
#endif