
let wasm;

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

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

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

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

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

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
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
function __wbg_adapter_18(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha6266361fc884972(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_21(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc1b52d340060ec82(arg0, arg1);
}

function __wbg_adapter_24(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha6266361fc884972(arg0, arg1, addHeapObject(arg2));
}

/**
*/
export function start() {
    wasm.start();
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

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        var ret = false;
        return ret;
    };
    imports.wbg.__wbg_self_1c83eb4471d9eb9b = handleError(function() {
        var ret = self.self;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_static_accessor_MODULE_abf5ae284bffdf45 = function() {
        var ret = module;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_require_5b2b5b594d809d9f = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).require(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_crypto_c12f14e810edcaa2 = function(arg0) {
        var ret = getObject(arg0).crypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_msCrypto_679be765111ba775 = function(arg0) {
        var ret = getObject(arg0).msCrypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        var ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_getRandomValues_05a60bf171bfc2be = function(arg0) {
        var ret = getObject(arg0).getRandomValues;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getRandomValues_3ac1b33c90b52596 = function(arg0, arg1, arg2) {
        getObject(arg0).getRandomValues(getArrayU8FromWasm0(arg1, arg2));
    };
    imports.wbg.__wbg_randomFillSync_6f956029658662ec = function(arg0, arg1, arg2) {
        getObject(arg0).randomFillSync(getArrayU8FromWasm0(arg1, arg2));
    };
    imports.wbg.__wbg_instanceof_Window_adf3196bdc02b386 = function(arg0) {
        var ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__wbg_document_6cc8d0b87c0a99b9 = function(arg0) {
        var ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_setInterval_06df6f1ebeafd66f = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    });
    imports.wbg.__wbg_getElementById_0cb6ad9511b1efc0 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_preventDefault_93d06688748bfc14 = function(arg0) {
        getObject(arg0).preventDefault();
    };
    imports.wbg.__wbg_addEventListener_9e7b0c3f65ebc0d7 = handleError(function(arg0, arg1, arg2, arg3) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    });
    imports.wbg.__wbg_setProperty_42eabadfcd7d6199 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    });
    imports.wbg.__wbg_setAttribute_727bdb9763037624 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    });
    imports.wbg.__wbg_debug_d101e002eb92f20b = function(arg0, arg1, arg2, arg3) {
        console.debug(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
    };
    imports.wbg.__wbg_error_7f083efc6bc6752c = function(arg0) {
        console.error(getObject(arg0));
    };
    imports.wbg.__wbg_error_cb872335132b1ef7 = function(arg0, arg1, arg2, arg3) {
        console.error(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
    };
    imports.wbg.__wbg_info_a25afde0ff8cd04a = function(arg0, arg1, arg2, arg3) {
        console.info(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
    };
    imports.wbg.__wbg_log_64f566ae90a6c43c = function(arg0, arg1, arg2, arg3) {
        console.log(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
    };
    imports.wbg.__wbg_warn_f632d7d3f55682b6 = function(arg0, arg1, arg2, arg3) {
        console.warn(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
    };
    imports.wbg.__wbg_style_9a41d46c005f7596 = function(arg0) {
        var ret = getObject(arg0).style;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_focus_5f74cb245be46131 = handleError(function(arg0) {
        getObject(arg0).focus();
    });
    imports.wbg.__wbg_instanceof_CanvasRenderingContext2d_5b86ec94bce38d5b = function(arg0) {
        var ret = getObject(arg0) instanceof CanvasRenderingContext2D;
        return ret;
    };
    imports.wbg.__wbg_setglobalAlpha_b88eed33e546d000 = function(arg0, arg1) {
        getObject(arg0).globalAlpha = arg1;
    };
    imports.wbg.__wbg_setstrokeStyle_ab391a0f9102e10c = function(arg0, arg1) {
        getObject(arg0).strokeStyle = getObject(arg1);
    };
    imports.wbg.__wbg_setfillStyle_2da87acf76dcbbcb = function(arg0, arg1) {
        getObject(arg0).fillStyle = getObject(arg1);
    };
    imports.wbg.__wbg_setlineWidth_85798545cf8a1f9d = function(arg0, arg1) {
        getObject(arg0).lineWidth = arg1;
    };
    imports.wbg.__wbg_setfont_7fb9fae174edc509 = function(arg0, arg1, arg2) {
        getObject(arg0).font = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_settextAlign_ef9d091f8d27eff3 = function(arg0, arg1, arg2) {
        getObject(arg0).textAlign = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_settextBaseline_c2a0520f9cd40851 = function(arg0, arg1, arg2) {
        getObject(arg0).textBaseline = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_beginPath_dcfb4577d60a6d17 = function(arg0) {
        getObject(arg0).beginPath();
    };
    imports.wbg.__wbg_fill_f7685c1e23b55d83 = function(arg0) {
        getObject(arg0).fill();
    };
    imports.wbg.__wbg_stroke_e549268dd54abc00 = function(arg0) {
        getObject(arg0).stroke();
    };
    imports.wbg.__wbg_rect_26c4e290a5e0ecda = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).rect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_clearRect_d61bc1791ebc06b1 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).clearRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_fillRect_e9ad0b5dde70ab3b = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).fillRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_restore_be383cadf1440d72 = function(arg0) {
        getObject(arg0).restore();
    };
    imports.wbg.__wbg_save_6d43ca6041c1ddb6 = function(arg0) {
        getObject(arg0).save();
    };
    imports.wbg.__wbg_fillText_0df13d8d3d07f630 = handleError(function(arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).fillText(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5);
    });
    imports.wbg.__wbg_key_590d4d2a765d1b58 = function(arg0, arg1) {
        var ret = getObject(arg1).key;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlCanvasElement_4f5b5ec6cd53ccf3 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLCanvasElement;
        return ret;
    };
    imports.wbg.__wbg_setwidth_5f26a8ba9dbfa0d0 = function(arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    };
    imports.wbg.__wbg_setheight_70f62727aa9383c2 = function(arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    };
    imports.wbg.__wbg_getContext_37ca0870acb096d9 = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    });
    imports.wbg.__wbg_call_8e95613cc6524977 = handleError(function(arg0, arg1) {
        var ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        var ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newnoargs_f3b8a801d5d4b079 = function(arg0, arg1) {
        var ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_07b2f89e82ceb76d = handleError(function() {
        var ret = self.self;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_window_ba85d88572adc0dc = handleError(function() {
        var ret = window.window;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_globalThis_b9277fc37e201fe5 = handleError(function() {
        var ret = globalThis.globalThis;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_global_e16303fe83e1d57f = handleError(function() {
        var ret = global.global;
        return addHeapObject(ret);
    });
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        var ret = debugString(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };
    imports.wbg.__wbindgen_closure_wrapper112 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 27, __wbg_adapter_18);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper114 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 27, __wbg_adapter_21);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper116 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 27, __wbg_adapter_24);
        return addHeapObject(ret);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    wasm.__wbindgen_start();
    return wasm;
}

export default init;

