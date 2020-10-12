import { Connection, State, Renderer, RendererResources } from 'client';

async function run() {
  const peer = new RTCPeerConnection({
    iceServers: [{
      urls: ['stun:stun.l.google.com:19302'],
    }],
  });
  const channel = await Connection.connect(peer).catch(err => console.error(err));
  if (!channel) {
    return;
  }

  const state_buffer = await fetch('state')
    	.then((r) => r.arrayBuffer())
    	.then((e) => new Uint8Array(e));
  const state = State.from_raw(state_buffer, channel);

  const shader_2d = await fetch("public/shader.glsl").then(r => r.text());
  const shader_instanced = await fetch("public/instanced.glsl").then(r => r.text());
  const resources = new RendererResources(shader_2d, shader_instanced);

  const canvas = document.getElementById('canvas');
  const overlay = document.getElementById('overlay');
  const ctx = canvas.getContext('webgl');
  const renderer = new Renderer(ctx, resources, canvas.width, canvas.height);
  const overlay_ctx = overlay.getContext('2d');

  const resize = (canvas) => {
    canvas.width = document.body.clientWidth;
    canvas.height = document.body.clientHeight;
  };
  canvas.addEventListener('resize', (event) => {
    resize(canvas);
    renderer.resize(canvas.width, canvas.height);
  });
  overlay.addEventListener('resize', resize.bind(null, overlay));
  resize(canvas);
  renderer.resize(canvas.width, canvas.height);
  resize(overlay);

  overlay.addEventListener('contextmenu', (event) => {
    event.preventDefault();
  });

  let leftMouseDown = null;
  let rightMouseDown = null;
  overlay.addEventListener('mousedown', (event) => {
    if (event.button === 0) {
      leftMouseDown = [event.x, event.y];
    } else if (event.button === 2) {
      rightMouseDown = [event.x, event.y];
    }
  });

  overlay.addEventListener('wheel', (event) => {
    renderer.zoom(event.deltaY);
  });

  let mousePos = [0, 0];
  overlay.addEventListener('mousemove', (event) => {
    mousePos[0] = event.x;
    mousePos[1] = event.y;

    if (rightMouseDown) {
      renderer.move_camera(event.x - rightMouseDown[0], event.y - rightMouseDown[1]);
      rightMouseDown = [event.x, event.y];
    }
  });

  let massOptions = document.getElementById("masses");
  overlay.addEventListener('mouseup', (event) => {
    if (event.button == 0 && leftMouseDown) {
      let mass = parseInt(massOptions.selectedOptions[0].value);
      let down = renderer.screen_to_world(leftMouseDown[0], leftMouseDown[1]);
      let up = renderer.screen_to_world(event.x, event.y);
      state.mouse_click_event(down[0], down[1], mass, up[0], up[1]);
      leftMouseDown = null;
    } else if (event.button == 2) {
      rightMouseDown = null;
    }
  })

  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    renderer.render(state);
    
    const bodies = state.to_json();

    let fontSize = 36;
    let textIndex = 0;
    overlay_ctx.clearRect(0, 0, canvas.width, canvas.height);
    overlay_ctx.fillStyle = 'white';
    overlay_ctx.font = `${fontSize}px serif`;

    overlay_ctx.fillText(`FPS: ${state.latency_secs()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`FRAME: ${state.current_frame()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`TARGET: ${state.target_frame()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`PKT LOSS: ${state.packet_loss()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`HASH SUC: ${state.hash_successes()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`HASH FAIL: ${state.hash_failures()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`BODIES: ${bodies.length}`, 0, (++textIndex * fontSize));

    if (leftMouseDown) {
      overlay_ctx.strokeStyle = 'blue';
      overlay_ctx.beginPath();
      overlay_ctx.moveTo(leftMouseDown[0], leftMouseDown[1]);
      overlay_ctx.lineTo(mousePos[0], mousePos[1]);
      overlay_ctx.stroke();
    }

    requestAnimationFrame(loop);
  };
  loop();
}

run();
