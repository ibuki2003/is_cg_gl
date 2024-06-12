#version 300 es
precision mediump float;

void mainImage(out vec4 fragColor, in vec2 uv) {
    float distanceFromCenter = length(uv - vec2(0));

    if (distanceFromCenter < 0.1) {
        fragColor = vec4(0, 0, 0.5, 0);
    } else {
        fragColor = vec4(vec3(uv + 0.5, sin(uv.x * 90.0) / 2. + 0.5), 1.0);
    }
}

out vec4 color;
uniform vec2 iResolution;
void main(void) {
  mainImage(
    color,
    (gl_FragCoord.xy - iResolution.xy / 2.0) / min(iResolution.x, iResolution.y)
  );
}

