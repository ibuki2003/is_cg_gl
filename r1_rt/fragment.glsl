#version 300 es
precision mediump float;

uniform vec2 iResolution;

struct Ray
{
  vec3 org;
  vec3 dir;
};

struct Hit
{
  float distanceToHitpoint;
  vec3 normal;
};

const float FilmDistance = 3.0;

uniform vec3 cameraPos;
const vec3 CameraTo = vec3(0.0, 0.0, 0.0);
const vec3 CameraUp = vec3(0.0, 1.0, 0.0);

uniform vec3 vertices[1000];
uniform vec3 normals[1000];
uniform uvec3 indices[1000];
uniform uint numIndices;

float LargeFloat() { return 1e+6; }

// 正規直交基底を計算する関数の例
void createOrthoNormalBasis(
    vec3 from, vec3 to, vec3 up,
    out vec3 u, out vec3 v, out vec3 w, out vec3 e
    )
{
  e = from;

  w = normalize(to - from);
  u = normalize(cross(up, w));
  v = cross(w, u);
}

Ray generateCameraRay(
    vec2 pixelCoordinate
    )
{
  // 1. ピクセル座標をカメラ座標系に変換
  // -> now pixelCoordinate is **already** normalized.

  // 2. カメラパラメータからカメラ座標系の正規直交基底を計算。
  vec3 u, v, w, e;
  createOrthoNormalBasis(cameraPos, CameraTo, CameraUp, u, v, w, e);

  // 3. ピクセル座標を基底を用いてワールド座標系に変換
  vec3 dir = w + (pixelCoordinate.x * u + pixelCoordinate.y * v) / FilmDistance;

  // 4. カメラレイを計算。
  Ray ray;
  ray.org = e;
  ray.dir = normalize(dir);

  return ray;
}

bool intersectToSphere(
    vec3 center, float radius, Ray ray,
    inout Hit hit
    )
{
  vec3 oc = ray.org - center;
  float a = dot(ray.dir, ray.dir);
  float b = 2.0 * dot(oc, ray.dir);
  float c = dot(oc, oc) - radius * radius;

  if (b * b - 4.0 * a * c < 0.0) { return false; }

  float t = (-b - sqrt(b * b - 4.0 * a * c)) / (2.0 * a);
  if (t > 0.0) {
    if (t > hit.distanceToHitpoint) { return false; }
    hit.distanceToHitpoint = t;
    hit.normal = normalize(ray.org + t * ray.dir - center);
    return true;
  }

  return false;
}

bool intersectToTriangle(
    vec3 a, vec3 b, vec3 c,
    vec3 na, vec3 nb, vec3 nc,
    Ray ray, inout Hit hit) {
  float t0 = 0.0;
  t0 += (a.x - b.x) * (a.y - c.y) * (ray.dir.z);
  t0 += (a.y - b.y) * (a.z - c.z) * (ray.dir.x);
  t0 += (a.z - b.z) * (a.x - c.x) * (ray.dir.y);
  t0 -= (a.x - b.x) * (a.z - c.z) * (ray.dir.y);
  t0 -= (a.y - b.y) * (a.x - c.x) * (ray.dir.z);
  t0 -= (a.z - b.z) * (a.y - c.y) * (ray.dir.x);
  if (t0 == 0.0) { return false; }

  float tt = 0.0;
  tt += (a.x - b.x) * (a.y - c.y) * (a.z - ray.org.z);
  tt += (a.y - b.y) * (a.z - c.z) * (a.x - ray.org.x);
  tt += (a.z - b.z) * (a.x - c.x) * (a.y - ray.org.y);
  tt -= (a.x - b.x) * (a.z - c.z) * (a.y - ray.org.y);
  tt -= (a.y - b.y) * (a.x - c.x) * (a.z - ray.org.z);
  tt -= (a.z - b.z) * (a.y - c.y) * (a.x - ray.org.x);

  float tb = 0.0;
  tb += (a.x - ray.org.x) * (a.y - c.y) * (ray.dir.z);
  tb += (a.y - ray.org.y) * (a.z - c.z) * (ray.dir.x);
  tb += (a.z - ray.org.z) * (a.x - c.x) * (ray.dir.y);
  tb -= (a.x - ray.org.x) * (a.z - c.z) * (ray.dir.y);
  tb -= (a.y - ray.org.y) * (a.x - c.x) * (ray.dir.z);
  tb -= (a.z - ray.org.z) * (a.y - c.y) * (ray.dir.x);

  float tc = 0.0;
  tc += (a.x - b.x) * (a.y - ray.org.y) * (ray.dir.z);
  tc += (a.y - b.y) * (a.z - ray.org.z) * (ray.dir.x);
  tc += (a.z - b.z) * (a.x - ray.org.x) * (ray.dir.y);
  tc -= (a.x - b.x) * (a.z - ray.org.z) * (ray.dir.y);
  tc -= (a.y - b.y) * (a.x - ray.org.x) * (ray.dir.z);
  tc -= (a.z - b.z) * (a.y - ray.org.y) * (ray.dir.x);

  float t = tt / t0;
  float beta = tb / t0;
  float gamma = tc / t0;
  float alpha = 1.0 - beta - gamma;
  if (
      (t < 0.0 || hit.distanceToHitpoint < t) ||
      (beta < 0.0 || beta > 1.0) ||
      (gamma < 0.0 || gamma > 1.0) ||
      (alpha < 0.0 || alpha > 1.0)
     ) { return false; }

  hit.distanceToHitpoint = t;
  hit.normal = alpha * na + beta * nb + gamma * nc;
  return true;
}


bool intersect(Ray ray, out Hit hit)
{
  hit.distanceToHitpoint = LargeFloat();

  intersectToSphere(vec3(0.0, 0.0, 0.1), 0.1, ray, hit);
  for (uint i = 0u; i < numIndices; i++) {
    intersectToTriangle(
        vertices[indices[i].x],
        vertices[indices[i].y],
        vertices[indices[i].z],
        normals[indices[i].x],
        normals[indices[i].y],
        normals[indices[i].z],
        ray, hit);
  }

  return hit.distanceToHitpoint < LargeFloat();
}

vec3 shade(Ray ray, Hit hit)
{
  return vec3(0.1 - dot(hit.normal, ray.dir) * 0.9);
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
  Ray ray = generateCameraRay(fragCoord);

  Hit hit;
  if (intersect(ray, hit)) {
    fragColor = vec4(shade(ray, hit), 1.0);
  } else {
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  }
}

out vec4 color;
void main(void) {
  mainImage(
    color,
    (gl_FragCoord.xy - iResolution.xy / 2.0) / min(iResolution.x, iResolution.y)
  );
}

