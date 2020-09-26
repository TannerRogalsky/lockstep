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
  const ctx = canvas.getContext('2d');

  canvas.addEventListener('mousedown', (event) => {
    state.mouse_down(event.x, event.y, Math.random() * 2000);
  });

  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    const json = state.to_json();
    canvas.width = document.body.clientWidth;
    canvas.height = document.body.clientHeight;

    ctx.font = '50px serif';
    ctx.fillText(`FPS: ${state.latency_secs()}`, 0, 50);
    ctx.fillText(`PKT LOSS: ${state.packet_loss()}`, 0, 100);
    ctx.fillText(`FRAME: ${json.frame_index}`, 0, 150);
    ctx.fillText(`BODIES: ${json.simulation.bodies.length}`, 0, 200);

    requestAnimationFrame(loop);
  };
  loop();
}

run();
