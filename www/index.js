import { Connection, State, InputState } from 'client';

async function run() {
  const peer = new RTCPeerConnection({
    iceServers: [{
      urls: ['stun:stun.l.google.com:19302'],
    }],
  });
  try {
    const channel = await Connection.connect(peer);
  } catch (err) {
    console.error(err);
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
    state.input_state_changed(new InputState(2 * (Math.random() - 0.5), 5, true));
  });

  canvas.addEventListener('mouseup', (event) => {
    state.input_state_changed(new InputState(0, 0, false));
  });

  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    // const json = state.to_json();
    requestAnimationFrame(loop);
  };
  loop();
}

run();
