<!-- ⚠️  THIS IS A GENERATED FILE -->
wasm-mt
=======

[Docs](https://docs.rs/wasm-mt) |
[GitHub](https://github.com/w3reality/wasm-mt) |
[Crate](https://crates.io/crates/wasm-mt)

[![crates][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![CI][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/wasm-mt.svg
[crates-url]: https://crates.io/crates/wasm-mt
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/w3reality/wasm-mt/blob/master/LICENSE-MIT
[actions-badge]: https://github.com/w3reality/wasm-mt/workflows/CI/badge.svg
[actions-url]: https://github.com/w3reality/wasm-mt/actions

A multithreading library for Rust and WebAssembly.

`wasm-mt` helps you create and execute Web Worker based threads. You can program the threads simply using Rust closures and orchestrate them with `async/await`.

#### Examples

- **`wasm-mt-pool`** - Thread pool library based on `wasm-mt`. [ [crate](https://crates.io/crates/wasm-mt-pool) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool) ]

You can run all the following apps in browser!

- **exec** - How to use <code>wasm_mt</code>. [ [live](https://w3reality.github.io/wasm-mt/examples/exec/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/examples/exec) ]
- **fib** - Computing a Fibonacci sequence with nested threads. [ [live](https://w3reality.github.io/wasm-mt/examples/fib/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/examples/fib) ]
- **executors** - Minimal serial/parallel executors using <code>wasm_mt</code>. [ [live](https://w3reality.github.io/wasm-mt/examples/executors/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/examples/executors) ]
- **parallel** - Julia set benchmark of serial/parallel executors. [ [live](https://w3reality.github.io/wasm-mt/examples/parallel/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/examples/parallel) ]
- **arraybuffers** - Demo of using <code>WasmMt::new_with_arraybuffers()</code>. [ [live](https://w3reality.github.io/wasm-mt/examples/arraybuffers/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/examples/arraybuffers) ]

#### Background and implementation

The preceding seminal work entitled ["Multithreading Rust and Wasm"](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html) by [@alexcrichton](https://github.com/alexcrichton) centers on [*Web Workers*](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API), *shared memory*, and [the WebAssembly threads proposal](https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md). Shared memory is built on top of [`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer) whose [availability across major browsers](https://caniuse.com/#feat=sharedarraybuffer) has been somewhat limited. Also, the rust-wasm thread implementation work, along with the threads proposal, seems still in progress.

On the contrary, Web Worker based multithreading in JavaScript has been [well supported for a long time](https://caniuse.com/#feat=webworkers). After experimenting, we have come up to a Rust ergonomic multithreading solution that does not require `SharedArrayBuffer`. It just works across all major browsers today and we named it `wasm-mt`.

Internally, we use the [`postMessage()`](https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage) Web Worker API (through bindings provided by [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen)) to initialize spawned threads. And, importantly, we keep using `postMessage()` for dynamically sending Rust closures (serialized by [`serde_traitobject`](https://github.com/alecmocatta/serde_traitobject)) to the spawned threads. By doing so, the parent thread can `await` the results of the closures executed in the spawned thread. We have found that this approach is highly flexible for extension, too. For example, it is straightforward to augment `WasmMt::Thread` to support more customized inter-thread communication patterns.

Note, however, that `wasm-mt` has some remarkable limitations compared to the ongoing shared memory based multithreading work led by `wasm-bindgen`. `wasm-mt` is not efficient in that it does **not include** support of the standard thread primitive operations:

- shared memory based message passing and mutexes,
- atomic instructions and efficient memory handling per [the threads proposal](https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md).

#### Thanks

- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) developers
- [@alecmocatta](https://github.com/alecmocatta) for the [serde_traitobject](https://github.com/alecmocatta/serde_traitobject) crate
- [swc-project](https://github.com/swc-project) that facilitates the [wasm-mt-test](https://github.com/w3reality/wasm-mt/tree/master/crates/test) crate

# Getting started

Requirements:

- rustc (nightly)
- [`wasm-pack build`](https://github.com/rustwasm/wasm-pack#%EF%B8%8F-commands) with the [`--target no-modules`](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#without-a-bundler) option

Cargo.toml:

```toml
wasm-mt = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_closure = "0.2"
```

# Creating a thread

First, create a [`WasmMt`] thread builder with [`new`][WasmMt::new] and initialize it:

```rust
use wasm_mt::prelude::*;

let pkg_js = "./pkg/exec.js"; // path to `wasm-bindgen`'s JS binding
let mt = WasmMt::new(pkg_js).and_init().await.unwrap();
```

Then, create a [`wasm_mt::Thread`][Thread] with the [`thread`][WasmMt::thread] function and initialize it:

```rust
let th = mt.thread().and_init().await.unwrap();
```

# Executing a thread

Using the [`exec!`] macro, you can execute a closure in the thread and `await` the result:

```rust
// fn add(a: i32, b: i32) -> i32 { a + b }

let a = 1;
let b = 2;
let ans = exec!(th, move || {
    let c = add(a, b);

    Ok(JsValue::from(c))
}).await?;
assert_eq!(ans, JsValue::from(3));
```

You can also execute an [async closure] with `exec!`:

```rust
// use wasm_mt::utils::sleep;
// async fn sub(a: i32, b: i32) -> i32 {
//    sleep(1000).await;
//    a - b
// }

let a = 1;
let b = 2;
let ans = exec!(th, async move || {
    let c = sub(a, b).await;

    Ok(JsValue::from(c))
}).await?;
assert_eq!(ans, JsValue::from(-1));
```

# Executing JavaScript in a thread

Using the [`exec_js!`] macro, you can execute JavaScript within a thread:

```rust
let ans = exec_js!(th, "
    const add = (a, b) => a + b;
    return add(1, 2);
").await?;
assert_eq!(ans, JsValue::from(3));
```

Similarly, use [`exec_js_async!`] for running asynchronous JavaScript:

```rust
let ans = exec_js_async!(th, "
    const sub = (a, b) => new Promise(resolve => {
        setTimeout(() => resolve(a - b), 1000);
    });
    return await sub(1, 2);
").await?;
assert_eq!(ans, JsValue::from(-1));
```

# Making executors

By using [`wasm_mt:Thread`][Thread], you can easily create custom executors. One such example is the [`wasm-mt-pool` crate](https://crates.io/crates/wasm-mt-pool). It provides a [thread pool](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#improving-throughput-with-a-thread-pool) that is based on the [work stealing] scheduling strategy.

Here, for simplicity, we show the implementation of much more  straightforward executors: a serial executor and a parallel executor.

First, prepare a `Vec<wasm_mt::Thread>` containing initialized threads:

```rust
let mut v: Vec<wasm_mt::Thread> = vec![];
for i in 0..4 {
    let th = mt.thread().and_init().await?;
    v.push(th);
}
```

Then, here's the executors in action. Note, in the latter case, we are using [`wasm_bindgen_futures::spawn_local`](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.spawn_local.html) to dispatch the threads in parallel.

```rust
console_ln!("🚀 serial executor:");
for th in &v {
    console_ln!("starting a thread");
    let ans = exec!(th, move || Ok(JsValue::from(42))).await?;
    console_ln!("ans: {:?}", ans);
}

console_ln!("🚀 parallel executor:");
for th in v {
    spawn_local(async move {
        console_ln!("starting a thread");
        let ans = exec!(th, move || Ok(JsValue::from(42))).await.unwrap();
        console_ln!("ans: {:?}", ans);
    });
}
```

Observe the starting/ending timing of each thread in the developer console:

```text
🚀 serial executor:
starting a thread
ans: JsValue(42)
starting a thread
ans: JsValue(42)
starting a thread
ans: JsValue(42)
starting a thread
ans: JsValue(42)
🚀 parallel executor:
(4) starting a thread
(4) ans: JsValue(42)
```

[async closure]: https://github.com/rust-lang/rfcs/blob/master/text/2394-async_await.md#async--closures
[work stealing]: https://en.wikipedia.org/wiki/Work_stealing
