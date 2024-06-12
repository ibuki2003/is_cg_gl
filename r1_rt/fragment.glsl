#version 300 es
precision mediump float;

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

// 各種パラメータの例
float FilmWidth() { return iResolution.x / 100.0; }
float FilmHeight() { return iResolution.y / 100.0;  }
float FilmDistance() { return 8.0; }

vec3 CameraFrom() { return vec3(5.0, 2.0, 3.0); }
vec3 CameraTo() { return vec3(0.2, 0.7, 0.2); }
vec3 CameraUp() { return vec3(0.0, 1.0, 0.0); }

float LargeFloat() { return 1e+6; }

// 正規直交基底を計算する関数の例
void createOrthoNormalBasis(
    vec3 from, vec3 to, vec3 up,
    out vec3 u, out vec3 v, out vec3 w, out vec3 e
    )
{
  // TODO: ベクトル正規化normalize()や外積cross()を用いて実装する。
}

vec3 convertToCameraCoordinateSystem(
    vec2 pixelCoordinate
    )
{
  // TODO: ピクセル座標をカメラ座標系に変換する。
  return vec3(0.0, 0.0, 0.0);
}

Ray generateCameraRay(
    vec2 pixelCoordinate
    )
{
  // TODO: 以下を実装する。
  // 1. ピクセル座標をカメラ座標系に変換
  // 2. カメラパラメータからカメラ座標系の正規直交基底を計算。
  // 3. ピクセル座標を基底を用いてワールド座標系に変換
  // 4. カメラレイを計算。
  Ray dummy;
  return dummy;
}

bool intersectToSphere(
    vec3 center, float radius, Ray ray,
    out Hit hit
    )
{
  // TODO: レイと球の交差判定を実装する。
  // 二次方程式の解の計算に帰着する。
  return false;
}

bool intersect(Ray ray, out Hit hit)
{
  hit.distanceToHitpoint = LargeFloat();

  // TODO: intersectToSphere を用いて具体的な球との交差判定を行う。

  return hit.distanceToHitpoint < LargeFloat();
}

vec3 shade(Ray ray, Hit hit)
{
  // TODO: なんらかのシェーディングを行う。
  return vec3(1.0, 1.0, 1.0);
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
  Ray ray = generateCameraRay(fragCoord);

  Hit hit;
  if (intersect(ray, hit))
  {
    fragColor = vec4(shade(ray, hit), 0.0);
  }
  else
  {
    fragColor = vec4(0.0);
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

