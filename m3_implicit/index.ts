import init, {start, draw, pan, update_spl } from 'wasm/m3_implicit'
(async () => {
  await init()

  const e = document.getElementById('canvas') as HTMLCanvasElement;
  start(e);

  e.addEventListener('mousemove', (event) => {
    if (event.buttons === 1) {
      pan(event.movementX, event.movementY, 0);
    }
  });
  e.addEventListener('wheel', (event) => {
    pan(0, 0, event.deltaY / 100);
  });

  const n_inp = document.getElementById('n') as HTMLInputElement;
  n_inp.addEventListener('change', (event) => {
    let v = parseInt(n_inp.value);
    v = Math.max(2, Math.min(100, v));
    update_spl(v);
  });
  update_spl(10);


  const drawFrame = () => {
    draw();
    requestAnimationFrame(drawFrame);
  };
  requestAnimationFrame(drawFrame);
})()
