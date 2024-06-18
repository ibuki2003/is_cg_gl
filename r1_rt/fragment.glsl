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

const vec3 CameraFrom = vec3(5.0, 2.0, 3.0);
const vec3 CameraTo = vec3(0.0, 0.0, 0.0);
const vec3 CameraUp = vec3(0.0, 1.0, 0.0);

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
  createOrthoNormalBasis(CameraFrom, CameraTo, CameraUp, u, v, w, e);

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

bool intersect(Ray ray, out Hit hit)
{
  hit.distanceToHitpoint = LargeFloat();

  intersectToSphere(vec3(0.0, 0.0, 0.0), 0.5, ray, hit);
  intersectToSphere(vec3(0.0, 0.0, 0.5), 0.5, ray, hit);

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

