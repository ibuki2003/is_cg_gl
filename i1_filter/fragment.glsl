#version 300 es
precision mediump float;

uniform vec2 iResolution;

uniform uint numIndices;

uniform float sigma_s;
uniform float sigma_r;

in vec2 v_texcoord;

float gaussian(float x, float sigma) {
  return exp(-x * x / (2.0 * sigma * sigma));
}

uniform sampler2D u_texture;

out vec4 color;

// bilateral filter
void main(void) {
  int r = int(ceil(3.0 * sigma_s));

  vec4 sum = vec4(0.0);
  float wsum = 0.0;

  vec4 center = texture(u_texture, v_texcoord);

  for (int x = -r; x <= r; x++) {
    for (int y = -r; y <= r; y++) {
      vec4 px = texture(u_texture, v_texcoord + vec2(float(x) / iResolution.x, float(y) / iResolution.y));
      float w = gaussian(float(x), sigma_s)
        * gaussian(float(y), sigma_s)
        * gaussian(length(px - center), sigma_r);
      sum += px * w;
      wsum += w;
    }
  }
  color = sum / wsum;
}

