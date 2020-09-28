import { Connection, State } from 'client';

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
  console.log(peer.sctp);

  const state_buffer = await fetch('state')
    	.then((r) => r.arrayBuffer())
    	.then((e) => new Uint8Array(e));
  const state = State.from_raw(state_buffer, channel);

  const canvas = document.getElementById('canvas');
  const overlay = document.getElementById('overlay');
  const ctx = canvas.getContext('2d');
  const overlay_ctx = overlay.getContext('2d');

  const resize = (canvas) => {
    canvas.width = document.body.clientWidth;
    canvas.height = document.body.clientHeight;
  };
  canvas.addEventListener('resize', resize.bind(null, canvas));
  overlay.addEventListener('resize', resize.bind(null, overlay));
  resize(canvas);
  resize(overlay);

  overlay.addEventListener('contextmenu', (event) => {
    event.preventDefault();
  });

  let mouseDown = null;
  overlay.addEventListener('mousedown', (event) => {
    if (event.button == 0) {
      mouseDown = [event.x, event.y];
    }
  });

  let mousePos = [0, 0];
  overlay.addEventListener('mousemove', (event) => {
    mousePos[0] = event.x;
    mousePos[1] = event.y;
  });

  let massOptions = document.getElementById("masses");
  overlay.addEventListener('mouseup', (event) => {
    if (!mouseDown) {
      return;
    }

    if (event.button == 0) {
      let mass = parseInt(massOptions.selectedOptions[0].value);
      state.mouse_click_event(mouseDown[0], mouseDown[1], mass, event.x, event.y);
      mouseDown = null;
    }
  })

  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    const bodies = state.to_json();

    for (const body of bodies) {
      ctx.fillStyle = 'red';
      ctx.beginPath();
      ctx.ellipse(body.x, body.y, body.radius, body.radius, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = 'black';
      ctx.stroke();
    }

    let fontSize = 50;
    let textIndex = 0;
    overlay_ctx.clearRect(0, 0, canvas.width, canvas.height);
    overlay_ctx.fillStyle = 'black';
    overlay_ctx.font = `${50}px serif`;

    overlay_ctx.fillText(`FPS: ${state.latency_secs()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`FRAME: ${state.current_frame()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`TARGET: ${state.target_frame()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`PKT LOSS: ${state.packet_loss()}`, 0, (++textIndex * fontSize));
    overlay_ctx.fillText(`BODIES: ${bodies.length}`, 0, (++textIndex * fontSize));

    if (mouseDown) {
      overlay_ctx.strokeStyle = 'blue';
      overlay_ctx.beginPath();
      overlay_ctx.moveTo(mouseDown[0], mouseDown[1]);
      overlay_ctx.lineTo(mousePos[0], mousePos[1]);
      overlay_ctx.stroke();
    }

    requestAnimationFrame(loop);
  };
  loop();
}

run();
