varying vec4 vColor;
varying vec2 vUV;

#ifdef VERTEX
attribute vec4 position;
attribute vec4 color;
attribute vec2 uv;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;

void main() {
    vColor = color;
    vUV = uv;
    gl_Position = uProjection * uView * uModel * position;
}
#endif

#ifdef FRAGMENT
uniform sampler2D tex0;
uniform vec4 uColor;

void main() {
//    fragColor = Texel(tex0, vUV) * vColor * uColor;
    fragColor = vec4(1., 1., 1., 1.);
}
#endif