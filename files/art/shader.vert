attribute vec3 aPosition;
attribute vec2 aTexCoord;
varying vec2 vTexCoord;

void main() {
    vTexCoord = aTexCoord;
    vec4 newPos = vec4(aPosition, 1.0);
    newPos.xy = newPos.xy * 2.0 - 1.0;
    gl_Position = newPos;
}