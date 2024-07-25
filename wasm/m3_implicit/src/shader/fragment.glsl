#version 300 es

precision highp float;

in vec4 vertexColor;
in vec3 vertexPos;
out vec4 fragmentColor;

//法線ベクトルを求める魔法の関数
vec3 getNormal ( vec3 position ) {
    vec3 dx = dFdx( position );
    vec3 dy = dFdy( position );
    return normalize( cross(dx, dy) );
}

void main() {
  vec3 normal = getNormal( vertexPos );
  float intensity = dot( normal, normalize( vec3(0.0, 0.0, 1.0) ) );
  intensity = clamp( intensity, 0.1, 1.0 );
  fragmentColor = vertexColor * vec4( intensity, intensity, intensity, 1.0 );
}
