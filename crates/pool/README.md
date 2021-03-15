<!-- ⚠️  THIS IS A GENERATED FILE -->
wasm-mt-pool
============

[Docs](https://docs.rs/wasm-mt-pool) |
[GitHub](https://github.com/w3reality/wasm-mt/tree/master/crates/pool) |
[Crate](https://crates.io/crates/wasm-mt-pool)

[![crates][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![CI][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/wasm-mt-pool.svg
[crates-url]: https://crates.io/crates/wasm-mt-pool
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/w3reality/wasm-mt/blob/master/crates/pool/LICENSE-MIT
[actions-badge]: https://github.com/w3reality/wasm-mt/workflows/CI/badge.svg
[actions-url]: https://github.com/w3reality/wasm-mt/actions

A thread pool library based on `wasm-mt` ([github](https://github.com/w3reality/wasm-mt) | [crate](https://crates.io/crates/wasm-mt)).

#### Examples

You can run all the following apps in browser!

- **pool_exec** - How to use <code>wasm_mt_pool</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/pool_exec/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/pool_exec) ]
- **http** - A multithreaded server based on <code>wasm_mt_pool</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/http/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/http) ]
- **pool_arraybuffers** - Demo of using <code>ThreadPool::new_with_arraybuffers()</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/pool_arraybuffers/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/pool_arraybuffers) ]

# Getting started

Requirements:

- rustc (nightly)
- [`wasm-pack build`](https://github.com/rustwasm/wasm-pack#%EF%B8%8F-commands) with the [`--target no-modules`](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#without-a-bundler) option

Cargo.toml:

```toml
wasm-mt-pool = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_closure = "0.2"
```

# Usage

```rust
#![feature(async_closure)]

use wasm_mt_pool::prelude::*;
use wasm_mt::utils::{console_ln, sleep};

let size = 2;
let pkg_js = "./pkg/pool_exec.js"; // path to `wasm-bindgen`'s JS binding
let pool = ThreadPool::new(size, pkg_js).and_init().await.unwrap();

let num = 4;

console_ln!("a) 💦 pool_exec! {} closures:", num);
for _ in 0..num {
    pool_exec!(pool, move || {
        console_ln!("a) closure: done.");
        Ok(JsValue::NULL)
    });
}

console_ln!("b) 💦 pool_exec! {} async closures:", num);
for _ in 0..num {
    pool_exec!(pool, async move || {
        sleep(1000).await;
        console_ln!("b) async closure: done.");
        Ok(JsValue::NULL)
    });
}

let cb = move |result| {
    console_ln!("callback: result: {:?}", result);
};

console_ln!("c) 💦 pool_exec! {} closures with callback:", num);
for _ in 0..num {
    pool_exec!(pool, move || {
        console_ln!("c) closure: done.");
        Ok(JsValue::from("C"))
    }, cb);
}

console_ln!("d) 💦 pool_exec! {} async closures with callback:", num);
for _ in 0..num {
    pool_exec!(pool, async move || {
        sleep(1000).await;
        console_ln!("d) async closure: done.");
        Ok(JsValue::from("D"))
    }, cb);
}

sleep(6_000).await; // Do sleep long enough to ensure all jobs are completed.
assert_eq!(pool.count_pending_jobs(), 0);
```
