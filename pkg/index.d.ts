/* tslint:disable */
/* eslint-disable */
/**
*/
export function main(): void;
/**
*/
export class Connection {
  free(): void;
/**
* @param {RTCPeerConnection} peer
* @returns {any}
*/
  static connect(peer: RTCPeerConnection): any;
/**
* @param {number} num
*/
  send_num(num: number): void;
/**
* @param {string} s
*/
  send_str(s: string): void;
/**
* @param {Uint8Array} data
*/
  send(data: Uint8Array): void;
/**
* @returns {Uint8Array | undefined}
*/
  recv(): Uint8Array | undefined;
/**
* @returns {RecvFuture}
*/
  recv_fut(): RecvFuture;
}
/**
*/
export class InputState {
  free(): void;
/**
* @param {number} mouse_x
* @param {number} mouse_y
* @param {boolean} mouse_down
*/
  constructor(mouse_x: number, mouse_y: number, mouse_down: boolean);
/**
* @returns {boolean}
*/
  mouse_down: boolean;
/**
* @returns {number}
*/
  mouse_x: number;
/**
* @returns {number}
*/
  mouse_y: number;
}
/**
*/
export class RecvFuture {
  free(): void;
/**
* @returns {any}
*/
  await(): any;
}
/**
*/
export class State {
  free(): void;
/**
* @param {Connection} connection
*/
  constructor(connection: Connection);
/**
* @param {Uint8Array} data
* @param {Connection} connection
* @returns {State}
*/
  static with_physics_raw(data: Uint8Array, connection: Connection): State;
/**
*/
  step(): void;
/**
* @param {InputState} input_state
*/
  input_state_changed(input_state: InputState): void;
/**
* @returns {any}
*/
  to_json(): any;
/**
* @returns {number}
*/
  latency_secs(): number;
}
