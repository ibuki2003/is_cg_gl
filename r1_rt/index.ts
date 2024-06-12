import vertexShaderSource from './vertex.glsl?raw';
import fragmentShaderSource from './fragment.glsl?raw';

async function main() {
  const canvas = document.getElementById('canvas') as HTMLCanvasElement
  canvas.width = 800;
  canvas.height = 600;
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

  const uniform_resolution = gl.getUniformLocation(program, 'iResolution');
  gl.uniform2f(uniform_resolution, canvas.width, canvas.height);

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


  const draw = () => {
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clearDepth(1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawElements(gl.TRIANGLES, i_count, gl.UNSIGNED_SHORT, 0);
    gl.flush();
    requestAnimationFrame(draw);
  };
  requestAnimationFrame(draw);
}

main();

