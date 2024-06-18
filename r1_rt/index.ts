import vertexShaderSource from './vertex.glsl?raw';
import fragmentShaderSource from './fragment.glsl?raw';
import { loadSTLAsync } from "@amandaghassaei/stl-parser";

interface Mesh {
  vertices: Float32Array;
  normals: Float32Array;
  indices: Uint16Array;
}


async function main() {
  const canvas = document.getElementById('canvas') as HTMLCanvasElement
  const output = document.getElementById('output')!;
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

  {
    const uniform_vertices = gl.getUniformLocation(program, 'vertices');
    const uniform_indices = gl.getUniformLocation(program, 'indices');
    const uniform_numindices = gl.getUniformLocation(program, 'numIndices');
    const uniform_normals = gl.getUniformLocation(program, 'normals');

    const mesh = await generateMesh_STL('../suzanne.stl');

    const cnt = Math.floor(mesh.indices.length / 3);

    gl.uniform3fv(uniform_vertices, mesh.vertices);
    gl.uniform3fv(uniform_normals, mesh.normals);
    gl.uniform3uiv(uniform_indices, mesh.indices);
    gl.uniform1ui(uniform_numindices, cnt);
    console.log({uniform_vertices, uniform_indices, uniform_numindices});
    console.log(cnt);
  }

  const uniform_camerapos = gl.getUniformLocation(program, 'cameraPos');

  let cameraPitch = 0.0;
  let cameraYaw = 0.0;
  let cameraDist = 10.0;

  const updateCamera = () => {
    const cameraPos = [
      cameraDist * Math.cos(cameraPitch) * Math.sin(cameraYaw),
      cameraDist * Math.cos(cameraPitch) * Math.cos(cameraYaw),
      cameraDist * Math.sin(cameraPitch),
    ];

    gl.uniform3fv(uniform_camerapos, cameraPos);
  };

  canvas.addEventListener('wheel', (e) => {
    cameraDist += e.deltaY * 0.01;
    if (cameraDist < 1.0) cameraDist = 1.0;
    updateCamera();
    e.preventDefault();
    return false;
  });


  canvas.addEventListener('mousemove', (e) => {
    if (e.buttons !== 1) return;

    cameraPitch += e.movementY * 0.01;
    cameraYaw -= e.movementX * 0.01;

    if (cameraPitch > Math.PI / 2) cameraPitch = Math.PI / 2;
    if (cameraPitch < -Math.PI / 2) cameraPitch = -Math.PI / 2;

    updateCamera();
  });

  updateCamera();


  gl.uniform3f(uniform_resolution, canvas.width, canvas.height, 0);

  let count = 0;
  let sum = 0;

  let fpscnt = 0;
  setInterval(() => {
    output.innerText = `${fpscnt} FPS`;
    fpscnt = 0;
  }, 1000);

  const draw = () => {
    fpscnt += 1;
    const t = performance.now();
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clearDepth(1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawElements(gl.TRIANGLES, i_count, gl.UNSIGNED_SHORT, 0);
    gl.flush();

    const t2 = performance.now();
    sum += t2 - t; count += 1;
    if (count >= 100) {
      console.log(`render time: ${sum / count} ms`);
      count = 0; sum = 0;
    }
    requestAnimationFrame(draw);
  };
  requestAnimationFrame(draw);
}

main();

async function generateMesh_sphere(): Promise<Mesh> {
  const T = 10;
  const U = 12;
  const vertices = Float32Array.from([
    0.0, 0.0, -1.0,
    ...new Array(T * 2 - 1).fill(0).flatMap((_,i) => {
      let lat = Math.PI * (i - T + 1) / T / 2;
      return new Array(U).fill(0).flatMap((_,j) => {
        let lon = Math.PI * 2 * j / U;
        return [
          Math.cos(lat) * Math.cos(lon),
          Math.cos(lat) * Math.sin(lon),
          Math.sin(lat),
        ];
      });
    }),
    0.0, 0.0, 1.0,
  ]);

  const indices = Uint16Array.from([
    ...new Array(U).fill(0).flatMap((_,i) => [0, i + 1, (i + 1) % U + 1]),
    ...new Array(T * 2 - 2).fill(0).flatMap((_,i) => (
      new Array(U).fill(0).flatMap((_,j) => [
        i * U + j + 1,
        i * U + (j + 1) % U + 1,
        (i + 1) * U + j + 1,

        i * U + (j + 1) % U + 1,
        (i + 1) * U + (j + 1) % U + 1,
        (i + 1) * U + j + 1,
      ])
    )),
    ...new Array(U).fill(0).flatMap((_,i) => [
      1 + (T * 2 - 2) * U + i,
      1 + (T * 2 - 2) * U + (i + 1) % U,
      (T * 2 - 1) * U + 1,
    ]),
  ]);

  return {
    vertices,
    normals: vertices,
    indices,
  };
}

async function generateMesh_STL(url: string): Promise<Mesh> {
  const mesh = (await loadSTLAsync(url))
    .mergeVertices()
    .scaleVerticesToUnitBoundingBox();

  const {
    vertices,
    facesNormals,
    facesIndices
  } = mesh;
  const normals = Array.from({length: vertices.length}, () => [0., 0., 0.]);
  for (let i = 0; i < facesIndices.length; i+=3) {
    for (let j = 0; j < 3; ++j) {
      const idx = facesIndices[i+j];
      normals[idx][0] += facesNormals[i];
      normals[idx][1] += facesNormals[i+1];
      normals[idx][2] += facesNormals[i+2];
    }
  }

  const normals_ = new Float32Array(normals.flatMap(v => {
    const n = Math.sqrt(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]);
    return [v[0]/n, v[1]/n, v[2]/n];
  }));
  return {
    vertices: vertices,
    normals: normals_,
    indices: new Uint16Array(facesIndices),
  };
}

