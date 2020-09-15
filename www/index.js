// import * as wasm from "hello-wasm-pack";

// wasm.greet();
import { Connection, State, InputState } from 'client';

const NUM_PACKETS = 20;
const SEND_INTERVAL = 1;
const ADDRESS = '/new_rtc_session';

async function run() {
  const peer = new RTCPeerConnection({
    iceServers: [{
      urls: ['stun:stun.l.google.com:19302'],
    }],
  });
  const channel = await Connection.connect(peer);
  console.log(peer.sctp);

  const physics_state = await fetch('state')
    	.then((r) => r.arrayBuffer())
    	.then((e) => new Uint8Array(e));
  const state = State.with_physics_raw(physics_state, channel);

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

  const scale = 10;
  const [wx, wy] = [0.5 * -scale, 0.5 * scale];
  const [ww, wh] = [10, -10];
  let prev_t = performance.now();
  const loop = (t) => {
    state.step();
    const json = state.to_json();

    ctx.scale(canvas.width / scale, -canvas.height / scale);
    ctx.translate(-wx, -wy);

    ctx.clearRect(wx, wy, ww, wh);
    ctx.fillStyle = 'red';
    ctx.fillRect(wx, wy, ww, wh);
    for (const maybeItem of json.colliders.items) {
      if (maybeItem.Occupied) {
        const item = maybeItem.Occupied.value;
        const [x, y] = item.position.translation;

        ctx.fillStyle = 'blue';
        ctx.beginPath();

        if (item.shape.Cuboid) {
          const [w, h] = item.shape.Cuboid.half_extents;
          ctx.rect(x - w, y - h, w * 2, h * 2);
        } else if (item.shape.Ball) {
          const { radius } = item.shape.Ball;
          ctx.ellipse(x, y, radius, radius, 0, 0, 2 * Math.PI);
        }
        ctx.fill();
      }
    }

    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.font = '24px serif';
    ctx.fillStyle = 'green';
    ctx.fillText(`latency (secs): ${state.latency_secs()}`, 0, 24);
    ctx.fillText(`fps (secs): ${(t - prev_t) / 1000}`, 0, 24 * 2);
    prev_t = t;

    requestAnimationFrame(loop);
  };
  loop();
}

run();
