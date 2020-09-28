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
  const bg_canvas = document.getElementById('bg');
  const ctx = canvas.getContext('2d');
  const bg_ctx = canvas.getContext('2d');

  const resize = (canvas) => {
    canvas.width = document.body.clientWidth;
    canvas.height = document.body.clientHeight;
  };
  canvas.addEventListener('resize', resize.bind(null, canvas));
  resize(canvas);

  let massOptions = document.getElementById("masses");
  canvas.addEventListener('mousedown', (event) => {
    let mass = parseInt(massOptions.selectedOptions[0].value);
    state.mouse_down(event.x, event.y, mass);
  });

  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    const bodies = state.to_json();

    let fontSize = 50;
    let textIndex = 0;

    ctx.clearRect(0, 0, canvas.width, fontSize * 5);

    for (const body of bodies) {
      ctx.fillStyle = 'red';
      ctx.beginPath();
      ctx.ellipse(body.x, body.y, body.radius, body.radius, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = 'black';
      ctx.stroke();
    }

    ctx.fillStyle = 'black';
    ctx.font = `${50}px serif`;
    ctx.fillText(`FPS: ${state.latency_secs()}`, 0, (++textIndex * fontSize));
    ctx.fillText(`FRAME: ${state.current_frame()}`, 0, (++textIndex * fontSize));
    ctx.fillText(`TARGET: ${state.target_frame()}`, 0, (++textIndex * fontSize));
    ctx.fillText(`PKT LOSS: ${state.packet_loss()}`, 0, (++textIndex * fontSize));
    ctx.fillText(`BODIES: ${bodies.length}`, 0, (++textIndex * fontSize));

    requestAnimationFrame(loop);
  };
  loop();
}

run();
