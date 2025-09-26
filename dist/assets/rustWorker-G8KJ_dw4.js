(function () {
  "use strict";
  let o;
  function x(e) {
    const t = o.__externref_table_alloc();
    return o.__wbindgen_export_2.set(t, e), t;
  }
  function d(e, t) {
    try {
      return e.apply(this, t);
    } catch (n) {
      const r = x(n);
      o.__wbindgen_exn_store(r);
    }
  }
  let u = null;
  function E() {
    return (u === null || u.byteLength === 0) && (u = new Float32Array(o.memory.buffer)), u;
  }
  function R(e, t) {
    return (e = e >>> 0), E().subarray(e / 4, e / 4 + t);
  }
  function y(e) {
    const t = typeof e;
    if (t == "number" || t == "boolean" || e == null) return `${e}`;
    if (t == "string") return `"${e}"`;
    if (t == "symbol") {
      const i = e.description;
      return i == null ? "Symbol" : `Symbol(${i})`;
    }
    if (t == "function") {
      const i = e.name;
      return typeof i == "string" && i.length > 0 ? `Function(${i})` : "Function";
    }
    if (Array.isArray(e)) {
      const i = e.length;
      let c = "[";
      i > 0 && (c += y(e[0]));
      for (let s = 1; s < i; s++) c += ", " + y(e[s]);
      return (c += "]"), c;
    }
    const n = /\[object ([^\]]+)\]/.exec(toString.call(e));
    let r;
    if (n && n.length > 1) r = n[1];
    else return toString.call(e);
    if (r == "Object")
      try {
        return "Object(" + JSON.stringify(e) + ")";
      } catch {
        return "Object";
      }
    return e instanceof Error
      ? `${e.name}: ${e.message}
${e.stack}`
      : r;
  }
  let m = 0,
    b = null;
  function g() {
    return (b === null || b.byteLength === 0) && (b = new Uint8Array(o.memory.buffer)), b;
  }
  const l =
      typeof TextEncoder < "u"
        ? new TextEncoder("utf-8")
        : {
            encode: () => {
              throw Error("TextEncoder not available");
            },
          },
    T =
      typeof l.encodeInto == "function"
        ? function (e, t) {
            return l.encodeInto(e, t);
          }
        : function (e, t) {
            const n = l.encode(e);
            return t.set(n), { read: e.length, written: n.length };
          };
  function F(e, t, n) {
    if (n === void 0) {
      const f = l.encode(e),
        _ = t(f.length, 1) >>> 0;
      return (
        g()
          .subarray(_, _ + f.length)
          .set(f),
        (m = f.length),
        _
      );
    }
    let r = e.length,
      i = t(r, 1) >>> 0;
    const c = g();
    let s = 0;
    for (; s < r; s++) {
      const f = e.charCodeAt(s);
      if (f > 127) break;
      c[i + s] = f;
    }
    if (s !== r) {
      s !== 0 && (e = e.slice(s)), (i = n(i, r, (r = s + e.length * 3), 1) >>> 0);
      const f = g().subarray(i + s, i + r),
        _ = T(e, f);
      (s += _.written), (i = n(i, r, s, 1) >>> 0);
    }
    return (m = s), i;
  }
  let a = null;
  function p() {
    return (
      (a === null ||
        a.buffer.detached === !0 ||
        (a.buffer.detached === void 0 && a.buffer !== o.memory.buffer)) &&
        (a = new DataView(o.memory.buffer)),
      a
    );
  }
  let w =
    typeof TextDecoder < "u"
      ? new TextDecoder("utf-8", { ignoreBOM: !0, fatal: !0 })
      : {
          decode: () => {
            throw Error("TextDecoder not available");
          },
        };
  typeof TextDecoder < "u" && w.decode();
  const M = 2146435072;
  let h = 0;
  function W(e, t) {
    return (
      (h += t),
      h >= M &&
        ((w =
          typeof TextDecoder < "u"
            ? new TextDecoder("utf-8", { ignoreBOM: !0, fatal: !0 })
            : {
                decode: () => {
                  throw Error("TextDecoder not available");
                },
              }),
        w.decode(),
        (h = t)),
      w.decode(g().subarray(e, e + t))
    );
  }
  function A(e, t) {
    return (e = e >>> 0), W(e, t);
  }
  function B(e, t, n, r) {
    o.init_audio_thread(e, t, n, r);
  }
  function D() {
    o.start_audio_processing_loop();
  }
  typeof FinalizationRegistry > "u" ||
    new FinalizationRegistry((e) => o.__wbg_notedto_free(e >>> 0, 1)),
    typeof FinalizationRegistry > "u" ||
      new FinalizationRegistry((e) => o.__wbg_sampler_free(e >>> 0, 1));
  const O = new Set(["basic", "cors", "default"]);
  async function I(e, t) {
    if (typeof Response == "function" && e instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming == "function")
        try {
          return await WebAssembly.instantiateStreaming(e, t);
        } catch (r) {
          if (e.ok && O.has(e.type) && e.headers.get("Content-Type") !== "application/wasm")
            console.warn(
              "`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",
              r
            );
          else throw r;
        }
      const n = await e.arrayBuffer();
      return await WebAssembly.instantiate(n, t);
    } else {
      const n = await WebAssembly.instantiate(e, t);
      return n instanceof WebAssembly.Instance ? { instance: n, module: e } : n;
    }
  }
  function z() {
    const e = {};
    return (
      (e.wbg = {}),
      (e.wbg.__wbg_byteLength_342d45700f2ba8c0 = function (t) {
        return t.byteLength;
      }),
      (e.wbg.__wbg_getindex_2125868ce4915de5 = function (t, n) {
        return t[n >>> 0];
      }),
      (e.wbg.__wbg_length_51cc4f6f9470ed54 = function (t) {
        return t.length;
      }),
      (e.wbg.__wbg_load_258901c26e79bcbb = function () {
        return d(function (t, n) {
          return Atomics.load(t, n >>> 0);
        }, arguments);
      }),
      (e.wbg.__wbg_log_f3c04200b995730f = function (t) {
        console.log(t);
      }),
      (e.wbg.__wbg_new_39a1bf523411b1a1 = function (t) {
        return new Int32Array(t);
      }),
      (e.wbg.__wbg_new_9190433fb67ed635 = function (t) {
        return new Uint8Array(t);
      }),
      (e.wbg.__wbg_new_ebfebeb8cef5d5bc = function (t) {
        return new Float32Array(t);
      }),
      (e.wbg.__wbg_newfromslice_75e0c51c9a25c17d = function (t, n) {
        return new Float32Array(R(t, n));
      }),
      (e.wbg.__wbg_notify_52ff7baf760af81d = function () {
        return d(function (t, n) {
          return Atomics.notify(t, n >>> 0);
        }, arguments);
      }),
      (e.wbg.__wbg_set_30943aea73afe8af = function (t, n, r) {
        t.set(n, r >>> 0);
      }),
      (e.wbg.__wbg_store_fbcdb5aee2674dce = function () {
        return d(function (t, n, r) {
          return Atomics.store(t, n >>> 0, r);
        }, arguments);
      }),
      (e.wbg.__wbg_subarray_2a08812eb1c33042 = function (t, n, r) {
        return t.subarray(n >>> 0, r >>> 0);
      }),
      (e.wbg.__wbg_subarray_43b8b822d816c0bf = function (t, n, r) {
        return t.subarray(n >>> 0, r >>> 0);
      }),
      (e.wbg.__wbg_subarray_a219824899e59712 = function (t, n, r) {
        return t.subarray(n >>> 0, r >>> 0);
      }),
      (e.wbg.__wbg_wait_ce55b9b792390fb6 = function () {
        return d(function (t, n, r) {
          return Atomics.wait(t, n >>> 0, r);
        }, arguments);
      }),
      (e.wbg.__wbg_wbindgendebugstring_bb652b1bc2061b6d = function (t, n) {
        const r = y(n),
          i = F(r, o.__wbindgen_malloc, o.__wbindgen_realloc),
          c = m;
        p().setInt32(t + 4, c, !0), p().setInt32(t + 0, i, !0);
      }),
      (e.wbg.__wbg_wbindgenthrow_4c11a24fca429ccf = function (t, n) {
        throw new Error(A(t, n));
      }),
      (e.wbg.__wbindgen_cast_2241b6af4c4b2941 = function (t, n) {
        return A(t, n);
      }),
      (e.wbg.__wbindgen_init_externref_table = function () {
        const t = o.__wbindgen_export_2,
          n = t.grow(4);
        t.set(0, void 0),
          t.set(n + 0, void 0),
          t.set(n + 1, null),
          t.set(n + 2, !0),
          t.set(n + 3, !1);
      }),
      e
    );
  }
  function U(e, t) {
    return (
      (o = e.exports),
      (S.__wbindgen_wasm_module = t),
      (a = null),
      (u = null),
      (b = null),
      o.__wbindgen_start(),
      o
    );
  }
  async function S(e) {
    if (o !== void 0) return o;
    typeof e < "u" &&
      (Object.getPrototypeOf(e) === Object.prototype
        ? ({ module_or_path: e } = e)
        : console.warn(
            "using deprecated parameters for the initialization function; pass a single object instead"
          )),
      typeof e > "u" && (e = new URL("/assets/rust_synth_bg-47ipHZb9.wasm", self.location.href));
    const t = z();
    (typeof e == "string" ||
      (typeof Request == "function" && e instanceof Request) ||
      (typeof URL == "function" && e instanceof URL)) &&
      (e = fetch(e));
    const { instance: n, module: r } = await I(await e, t);
    return U(n, r);
  }
  (async () => {
    await S(),
      console.log("[RUST WORKER] Rust WASM ready in Worker!"),
      self.postMessage({ type: "module_end_init" });
  })(),
    (self.onmessage = (e) => {
      if (e.data.type === "init_wasm") {
        const t = e.data.sharedBuffer,
          n = e.data.midi_queue_buffer,
          r = e.data.ringBufferSize,
          i = e.data.osc_queue_buffer;
        if (
          !(t instanceof SharedArrayBuffer) ||
          typeof r != "number" ||
          !(n instanceof SharedArrayBuffer) ||
          !(i instanceof SharedArrayBuffer)
        ) {
          console.log(
            "error - invalid buffers:",
            "audio buffer valid:",
            t instanceof SharedArrayBuffer,
            "midi buffer valid:",
            n instanceof SharedArrayBuffer,
            "osc buffer valid:",
            i instanceof SharedArrayBuffer,
            "ring buffer size:",
            r
          );
          return;
        }
        new Int32Array(t, 0, 3).subarray(0, 1),
          B(t, r, n, i),
          console.log("[RUST WORKER] initialisation done, processing loop..."),
          D();
      }
    });
})();
