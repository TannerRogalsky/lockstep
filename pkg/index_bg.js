import * as wasm from './index_bg.wasm';

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_24(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c980ef67dcc2d9d(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_27(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hfcd0da4d5b74a838(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_30(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c980ef67dcc2d9d(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_33(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c980ef67dcc2d9d(arg0, arg1, addHeapObject(arg2));
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
*/
export function main() {
    wasm.main();
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

function handleError(f) {
    return function () {
        try {
            return f.apply(this, arguments);

        } catch (e) {
            wasm.__wbindgen_exn_store(addHeapObject(e));
        }
    };
}
function __wbg_adapter_135(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h36712187a371be70(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

/**
*/
export class Connection {

    static __wrap(ptr) {
        const obj = Object.create(Connection.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_connection_free(ptr);
    }
    /**
    * @param {RTCPeerConnection} peer
    * @returns {any}
    */
    static connect(peer) {
        var ret = wasm.connection_connect(addHeapObject(peer));
        return takeObject(ret);
    }
    /**
    * @param {number} num
    */
    send_num(num) {
        wasm.connection_send_num(this.ptr, num);
    }
    /**
    * @param {string} s
    */
    send_str(s) {
        var ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.connection_send(this.ptr, ptr0, len0);
    }
    /**
    * @param {Uint8Array} data
    */
    send(data) {
        var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.connection_send(this.ptr, ptr0, len0);
    }
    /**
    * @returns {Uint8Array | undefined}
    */
    recv() {
        try {
            const retptr = wasm.__wbindgen_export_5.value - 16;
            wasm.__wbindgen_export_5.value = retptr;
            wasm.connection_recv(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            let v0;
            if (r0 !== 0) {
                v0 = getArrayU8FromWasm0(r0, r1).slice();
                wasm.__wbindgen_free(r0, r1 * 1);
            }
            return v0;
        } finally {
            wasm.__wbindgen_export_5.value += 16;
        }
    }
    /**
    * @returns {RecvFuture}
    */
    recv_fut() {
        var ret = wasm.connection_recv_fut(this.ptr);
        return RecvFuture.__wrap(ret);
    }
}
/**
*/
export class InputState {

    static __wrap(ptr) {
        const obj = Object.create(InputState.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_inputstate_free(ptr);
    }
    /**
    * @returns {number}
    */
    get mouse_x() {
        var ret = wasm.__wbg_get_inputstate_mouse_x(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set mouse_x(arg0) {
        wasm.__wbg_set_inputstate_mouse_x(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get mouse_y() {
        var ret = wasm.__wbg_get_inputstate_mouse_y(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set mouse_y(arg0) {
        wasm.__wbg_set_inputstate_mouse_y(this.ptr, arg0);
    }
    /**
    * @returns {boolean}
    */
    get mouse_down() {
        var ret = wasm.__wbg_get_inputstate_mouse_down(this.ptr);
        return ret !== 0;
    }
    /**
    * @param {boolean} arg0
    */
    set mouse_down(arg0) {
        wasm.__wbg_set_inputstate_mouse_down(this.ptr, arg0);
    }
    /**
    * @param {number} mouse_x
    * @param {number} mouse_y
    * @param {boolean} mouse_down
    */
    constructor(mouse_x, mouse_y, mouse_down) {
        var ret = wasm.inputstate_new(mouse_x, mouse_y, mouse_down);
        return InputState.__wrap(ret);
    }
}
/**
*/
export class RecvFuture {

    static __wrap(ptr) {
        const obj = Object.create(RecvFuture.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_recvfuture_free(ptr);
    }
    /**
    * @returns {any}
    */
    await() {
        var ptr = this.ptr;
        this.ptr = 0;
        var ret = wasm.recvfuture_await(ptr);
        return takeObject(ret);
    }
}
/**
*/
export class State {

    static __wrap(ptr) {
        const obj = Object.create(State.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_state_free(ptr);
    }
    /**
    * @param {Connection} connection
    */
    constructor(connection) {
        _assertClass(connection, Connection);
        var ptr0 = connection.ptr;
        connection.ptr = 0;
        var ret = wasm.state_new(ptr0);
        return State.__wrap(ret);
    }
    /**
    * @param {Uint8Array} data
    * @param {Connection} connection
    * @returns {State}
    */
    static with_physics_raw(data, connection) {
        var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        _assertClass(connection, Connection);
        var ptr1 = connection.ptr;
        connection.ptr = 0;
        var ret = wasm.state_with_physics_raw(ptr0, len0, ptr1);
        return State.__wrap(ret);
    }
    /**
    */
    step() {
        wasm.state_step(this.ptr);
    }
    /**
    * @param {InputState} input_state
    */
    input_state_changed(input_state) {
        _assertClass(input_state, InputState);
        var ptr0 = input_state.ptr;
        input_state.ptr = 0;
        wasm.state_input_state_changed(this.ptr, ptr0);
    }
    /**
    * @returns {any}
    */
    to_json() {
        var ret = wasm.state_to_json(this.ptr);
        return takeObject(ret);
    }
    /**
    * @returns {number}
    */
    latency_secs() {
        var ret = wasm.state_latency_secs(this.ptr);
        return ret;
    }
}

export const __wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

export const __wbg_connection_new = function(arg0) {
    var ret = Connection.__wrap(arg0);
    return addHeapObject(ret);
};

export const __wbindgen_cb_drop = function(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    var ret = false;
    return ret;
};

export const __wbindgen_string_new = function(arg0, arg1) {
    var ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export const __wbindgen_object_clone_ref = function(arg0) {
    var ret = getObject(arg0);
    return addHeapObject(ret);
};

export const __wbg_new_68adb0d58759a4ed = function() {
    var ret = new Object();
    return addHeapObject(ret);
};

export const __wbindgen_number_new = function(arg0) {
    var ret = arg0;
    return addHeapObject(ret);
};

export const __wbg_set_2e79e744454afade = function(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
};

export const __wbg_new_59cb74e423758ede = function() {
    var ret = new Error();
    return addHeapObject(ret);
};

export const __wbg_stack_558ba5917b466edd = function(arg0, arg1) {
    var ret = getObject(arg1).stack;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_error_4bb6c2a97407129a = function(arg0, arg1) {
    try {
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(arg0, arg1);
    }
};

export const __wbg_instanceof_Window_e8f84259147dce74 = function(arg0) {
    var ret = getObject(arg0) instanceof Window;
    return ret;
};

export const __wbg_fetch_4875ac39fd69c38e = function(arg0, arg1) {
    var ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_debug_cd8a0aad17c8c92f = function(arg0, arg1, arg2, arg3) {
    console.debug(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_error_7dcc755846c00ef7 = function(arg0) {
    console.error(getObject(arg0));
};

export const __wbg_error_b47ee9a774776bfa = function(arg0, arg1, arg2, arg3) {
    console.error(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_info_0c64856d96c69122 = function(arg0, arg1, arg2, arg3) {
    console.info(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_log_7fc0936bf7223435 = function(arg0, arg1, arg2, arg3) {
    console.log(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_warn_f88df7e1e2a26187 = function(arg0, arg1, arg2, arg3) {
    console.warn(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_sdp_95a85c5e0b6ab274 = function(arg0, arg1) {
    var ret = getObject(arg1).sdp;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_addEventListener_116c561435e7160d = handleError(function(arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
});

export const __wbg_removeEventListener_d9ceb7fdf4ca5166 = handleError(function(arg0, arg1, arg2, arg3) {
    getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
});

export const __wbg_localDescription_94f1f18ac297f23a = function(arg0) {
    var ret = getObject(arg0).localDescription;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_addIceCandidate_3bb6335d6cc17f31 = function(arg0, arg1) {
    var ret = getObject(arg0).addIceCandidate(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createDataChannel_f76a8f472039a2d3 = function(arg0, arg1, arg2, arg3) {
    var ret = getObject(arg0).createDataChannel(getStringFromWasm0(arg1, arg2), getObject(arg3));
    return addHeapObject(ret);
};

export const __wbg_createOffer_c78d73179584888c = function(arg0) {
    var ret = getObject(arg0).createOffer();
    return addHeapObject(ret);
};

export const __wbg_setLocalDescription_002d93d6396d1bf4 = function(arg0, arg1) {
    var ret = getObject(arg0).setLocalDescription(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_setRemoteDescription_98ff53973c4e3c27 = function(arg0, arg1) {
    var ret = getObject(arg0).setRemoteDescription(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_instanceof_RtcDataChannel_377b23b33fe4a2c7 = function(arg0) {
    var ret = getObject(arg0) instanceof RTCDataChannel;
    return ret;
};

export const __wbg_setonopen_3ddbba5baeef9d6c = function(arg0, arg1) {
    getObject(arg0).onopen = getObject(arg1);
};

export const __wbg_setonerror_3f42a37005808b13 = function(arg0, arg1) {
    getObject(arg0).onerror = getObject(arg1);
};

export const __wbg_send_95125abb75ed8135 = handleError(function(arg0, arg1, arg2) {
    getObject(arg0).send(getArrayU8FromWasm0(arg1, arg2));
});

export const __wbg_instanceof_Response_df90672bc1607490 = function(arg0) {
    var ret = getObject(arg0) instanceof Response;
    return ret;
};

export const __wbg_json_6b2a36109345a9ee = handleError(function(arg0) {
    var ret = getObject(arg0).json();
    return addHeapObject(ret);
});

export const __wbg_now_acfa6ea53a7be2c2 = function(arg0) {
    var ret = getObject(arg0).now();
    return ret;
};

export const __wbg_data_6ea4600a7910f404 = function(arg0) {
    var ret = getObject(arg0).data;
    return addHeapObject(ret);
};

export const __wbg_candidate_b01a5c110991672f = function(arg0) {
    var ret = getObject(arg0).candidate;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_newwithstrandinit_b18f1bd8ba76e760 = handleError(function(arg0, arg1, arg2) {
    var ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
    return addHeapObject(ret);
});

export const __wbg_get_2e96a823c1c5a5bd = handleError(function(arg0, arg1) {
    var ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
});

export const __wbg_call_e9f0ce4da840ab94 = handleError(function(arg0, arg1) {
    var ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
});

export const __wbg_new_17534eac4df3cd22 = function() {
    var ret = new Array();
    return addHeapObject(ret);
};

export const __wbg_push_7114ccbf1c58e41f = function(arg0, arg1) {
    var ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export const __wbg_new_4896ab6bba55e0d9 = function(arg0, arg1) {
    var ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export const __wbg_newnoargs_e2fdfe2af14a2323 = function(arg0, arg1) {
    var ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export const __wbg_call_0dad7db75ec90ae7 = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
});

export const __wbg_new_8172f4fed77fdb7c = function() {
    var ret = new Object();
    return addHeapObject(ret);
};

export const __wbg_new_7039bf8b99f049e1 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_135(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        var ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
};

export const __wbg_resolve_4df26938859b92e3 = function(arg0) {
    var ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export const __wbg_then_ffb6e71f7a6735ad = function(arg0, arg1) {
    var ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_then_021fcdc7f0350b58 = function(arg0, arg1, arg2) {
    var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export const __wbg_self_179e8c2a5a4c73a3 = handleError(function() {
    var ret = self.self;
    return addHeapObject(ret);
});

export const __wbg_window_492cfe63a6e41dfa = handleError(function() {
    var ret = window.window;
    return addHeapObject(ret);
});

export const __wbg_globalThis_8ebfea75c2dd63ee = handleError(function() {
    var ret = globalThis.globalThis;
    return addHeapObject(ret);
});

export const __wbg_global_62ea2619f58bf94d = handleError(function() {
    var ret = global.global;
    return addHeapObject(ret);
});

export const __wbindgen_is_undefined = function(arg0) {
    var ret = getObject(arg0) === undefined;
    return ret;
};

export const __wbg_buffer_88f603259d7a7b82 = function(arg0) {
    var ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export const __wbg_newwithbyteoffsetandlength_a048d126789a272b = function(arg0, arg1, arg2) {
    var ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export const __wbg_length_2e98733d73dac355 = function(arg0) {
    var ret = getObject(arg0).length;
    return ret;
};

export const __wbg_new_85d8a1fc4384acef = function(arg0) {
    var ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

export const __wbg_set_478951586c457484 = function(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

export const __wbg_set_afe54b1eeb1aa77c = handleError(function(arg0, arg1, arg2) {
    var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
});

export const __wbindgen_string_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    var ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbindgen_debug_string = function(arg0, arg1) {
    var ret = debugString(getObject(arg1));
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export const __wbindgen_rethrow = function(arg0) {
    throw takeObject(arg0);
};

export const __wbindgen_memory = function() {
    var ret = wasm.memory;
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper712 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 160, __wbg_adapter_27);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper341 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 63, __wbg_adapter_24);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper343 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 63, __wbg_adapter_30);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper339 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 63, __wbg_adapter_33);
    return addHeapObject(ret);
};

