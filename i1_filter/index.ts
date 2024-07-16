import vertexShaderSource from './vertex.glsl?raw';
import fragmentShaderSource from './fragment.glsl?raw';

async function main() {
  const canvas = document.getElementById('canvas') as HTMLCanvasElement
  const output = document.getElementById('output')!;
  const input_file = document.getElementById('input_file') as HTMLInputElement;
  const origimg = document.getElementById('img_original') as HTMLImageElement;

  const input_sigma_s = document.getElementById('input_sigma_s') as HTMLInputElement;
  const input_sigma_r = document.getElementById('input_sigma_r') as HTMLInputElement;

  canvas.width = 256;
  canvas.height = 256;

  const gl = canvas.getContext('webgl2');
  if (gl === null) throw new Error('failed to get webgl2 context');

  // load from ./fragment.glsl
  const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
  if (fragmentShader === null) throw new Error('failed to create fragment shader');
  gl.shaderSource(fragmentShader, fragmentShaderSource);
  gl.compileShader(fragmentShader);
  if (!gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS))
    throw new Error('failed to compile fragment shader: ' + gl.getShaderInfoLog(fragmentShader));

  const vertexShader = gl.createShader(gl.VERTEX_SHADER);
  if (vertexShader === null) throw new Error('failed to create vertex shader');
  gl.shaderSource(vertexShader, vertexShaderSource);
  gl.compileShader(vertexShader);
  if (!gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS))
    throw new Error('failed to compile vertex shader: ' + gl.getShaderInfoLog(vertexShader));

  const program = gl.createProgram();
  if (program === null) throw new Error('failed to create program');
  gl.attachShader(program, fragmentShader)
  gl.attachShader(program, vertexShader)
  gl.linkProgram(program)
  if (!gl.getProgramParameter(program, gl.LINK_STATUS))
    throw new Error('failed to link program: ' + gl.getProgramInfoLog(program));

  gl.useProgram(program);

  const vao = gl.createVertexArray();
  if (vao === null) throw new Error('failed to create vertex array');
  gl.bindVertexArray(vao);

  const idx = Uint16Array.from([
      0, 1, 2,
      2, 1, 3,
  ]);
  const i_count = idx.length;
  const ibo = gl.createBuffer();
  if (ibo === null) throw new Error('failed to create index buffer');
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);

  const vertices = Float32Array.from([
      -1.0, -1.0, 0.0,
      1.0, -1.0, 0.0,
      -1.0, 1.0, 0.0,
      1.0, 1.0, 0.0,
  ]);
  const vbo = gl.createBuffer();
  if (vbo === null) throw new Error('failed to create vertex buffer');
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);

  gl.bindVertexArray(vao);

  gl.enable(gl.DEPTH_TEST);
  gl.depthFunc(gl.LEQUAL);
  gl.enable(gl.CULL_FACE);

  const uniform_sigma_s = gl.getUniformLocation(program, 'sigma_s');
  const uniform_sigma_r = gl.getUniformLocation(program, 'sigma_r');

  const uniform_resolution = gl.getUniformLocation(program, 'iResolution');
  gl.uniform2f(uniform_resolution, canvas.width, canvas.height);

  const texture = createTexture(gl, origimg);
  gl.bindTexture(gl.TEXTURE_2D, texture);

  const draw_raw = () => {
    console.log('draw');
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clearDepth(1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawElements(gl.TRIANGLES, i_count, gl.UNSIGNED_SHORT, 0);
    gl.flush();
  };

  let draw_debounce_timer: null | number = null;
  const draw = () => {
    if (draw_debounce_timer !== null) {
      clearTimeout(draw_debounce_timer);
    }
    draw_debounce_timer = setTimeout(() => {
      draw_raw();
      draw_debounce_timer = null;
    }, 100) as unknown as number;
  };

  // requestAnimationFrame(draw);

  const update_sigma = () => {
    const sigma_s = Math.max(0.1, parseFloat(input_sigma_s.value));
    const sigma_r = Math.max(0.01, parseFloat(input_sigma_r.value));
    console.log({sigma_r, sigma_s});
    gl.uniform1f(uniform_sigma_r, sigma_r);
    gl.uniform1f(uniform_sigma_s, sigma_s);
    draw();
  };
  input_sigma_s.addEventListener('change', update_sigma);
  input_sigma_r.addEventListener('change', update_sigma);
  update_sigma();

  origimg.addEventListener('load', () => {
    canvas.width = origimg.naturalWidth;
    canvas.height = origimg.naturalHeight;
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.uniform2f(uniform_resolution, canvas.width, canvas.height);
    draw();
  });

  const reader = new FileReader();
  reader.onload = async (e) => {
    const src = e.target?.result as string;
    origimg.src = src;
  };
  input_file.addEventListener('change', async (e) => {
    reader.readAsDataURL(input_file.files![0]);
  });
}

// original from: https://developer.mozilla.org/ja/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL

function createTexture(gl: WebGL2RenderingContext, image: HTMLImageElement) {
  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);

  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    1,
    1,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    new Uint8Array([0, 0, 255, 255]), // blue pixel
  );

  // image.onload = () => {
  image.addEventListener('load', () => {
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      image.naturalWidth,
      image.naturalHeight,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      image,
    );

    // WebGL1 は画像の大きさが 2 のべき乗であるかどうかで
    // 要求されるものが異なるので、画像の両方の軸が 2 の
    // べき乗かどうかを調べます。
    if (isPowerOf2(image.naturalWidth) && isPowerOf2(image.naturalHeight)) {
      // 2 のべき乗なので、 mips を作成します。
      gl.generateMipmap(gl.TEXTURE_2D);
    } else {
      // 2 のべき乗ではないので、 mips をオフにして、
      // エッジにクランプするようにラッピングを設定します。
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    }
  });

  return texture;
}

function isPowerOf2(value: number) {
  return (value & (value - 1)) === 0;
}

main();

