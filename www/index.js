import { Connection, State, InputState } from 'client';

async function run() {
  const peer = new RTCPeerConnection({
    iceServers: [{
      urls: ['stun:stun.l.google.com:19302'],
    }],
  });
  const channel = await Connection.connect(peer);
  console.log(peer.sctp);

  const state_buffer = await fetch('state')
    	.then((r) => r.arrayBuffer())
    	.then((e) => new Uint8Array(e));
  const state = State.from_raw(state_buffer, channel);

  // let i = 0;
  // let channelLoop = () => {
  //     channel.send_num(i);
  //     i++;
  //     if (i > 1) {
  //         return;
  //     }
  //     requestAnimationFrame(channelLoop);
  // };
  // channelLoop();

  // let p1 = channel.recv_fut().await();
  // let p2 = channel.recv_fut().await();
  // console.log(await p2);
  // console.log(await p1);

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
