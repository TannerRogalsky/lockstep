// import * as wasm from "hello-wasm-pack";

// wasm.greet();
// import { Connection, State, InputState } from 'client';
import { SharedState } from 'client';
import { Simulation, Body } from 'nbody';

async function run() {
  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  let simulation = new Simulation();
  for (var i = 0; i < 3; i++) {
    simulation.add_body(new Body(Math.random() * canvas.width, Math.random() * canvas.height, Math.random() * 20));
  }

  canvas.addEventListener('mousedown', (event) => {
    simulation.add_body(new Body(event.clientX, event.clientY, Math.random() * 20));
  });

  const loop = (t) => {
    console.log(t);
    simulation.step();
    ctx.fillStyle = 'blue';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = 'red';
    for (var i = 0; i < simulation.body_count(); i++) {
      let r = simulation.render_data(i);
      ctx.beginPath();
      ctx.ellipse(r.position_x, r.position_y, r.radius, r.radius, 0, 0, 2 * Math.PI);
      ctx.fill();
      r.free();
    }
  
    requestAnimationFrame(loop);
  }
  requestAnimationFrame(loop);
}

// async function run() {
//   const peer = new RTCPeerConnection({
//     iceServers: [{
//       urls: ['stun:stun.l.google.com:19302'],
//     }],
//   });
//   const channel = await Connection.connect(peer);
//   console.log(peer.sctp);

//   const physics_state = await fetch('state')
//     	.then((r) => r.arrayBuffer())
//     	.then((e) => new Uint8Array(e));
//   const state = State.with_physics_raw(physics_state, channel);

//   // let i = 0;
//   // let channelLoop = () => {
//   //     channel.send_num(i);
//   //     i++;
//   //     if (i > 1) {
//   //         return;
//   //     }
//   //     requestAnimationFrame(channelLoop);
//   // };
//   // channelLoop();

//   // let p1 = channel.recv_fut().await();
//   // let p2 = channel.recv_fut().await();
//   // console.log(await p2);
//   // console.log(await p1);

//   const canvas = document.getElementById('canvas');
//   const ctx = canvas.getContext('2d');

//   canvas.addEventListener('mousedown', (event) => {
//     state.input_state_changed(new InputState(2 * (Math.random() - 0.5), 5, true));
//   });

//   canvas.addEventListener('mouseup', (event) => {
//     state.input_state_changed(new InputState(0, 0, false));
//   });

//   const scale = 10;
//   const [wx, wy] = [0.5 * -scale, 0.5 * scale];
//   const [ww, wh] = [10, -10];
//   let prev_t = performance.now();
//   const loop = (t) => {
//     state.step();
//     const json = state.to_json();

//     ctx.scale(canvas.width / scale, -canvas.height / scale);
//     ctx.translate(-wx, -wy);

//     ctx.clearRect(wx, wy, ww, wh);
//     ctx.fillStyle = 'red';
//     ctx.fillRect(wx, wy, ww, wh);
//     for (const maybeItem of json.colliders.items) {
//       if (maybeItem.Occupied) {
//         const item = maybeItem.Occupied.value;
//         const [x, y] = item.position.translation;

//         ctx.fillStyle = 'blue';
//         ctx.beginPath();

//         if (item.shape.Cuboid) {
//           const [w, h] = item.shape.Cuboid.half_extents;
//           ctx.rect(x - w, y - h, w * 2, h * 2);
//         } else if (item.shape.Ball) {
//           const { radius } = item.shape.Ball;
//           ctx.ellipse(x, y, radius, radius, 0, 0, 2 * Math.PI);
//         }
//         ctx.fill();
//       }
//     }

//     ctx.setTransform(1, 0, 0, 1, 0, 0);
//     ctx.font = '24px serif';
//     ctx.fillStyle = 'green';
//     ctx.fillText(`latency (secs): ${state.latency_secs()}`, 0, 24);
//     ctx.fillText(`fps (secs): ${(t - prev_t) / 1000}`, 0, 24 * 2);
//     prev_t = t;

//     requestAnimationFrame(loop);
//   };
//   loop();
// }

run();
