wasm-mt-pool
============

[Docs](https://docs.rs/wasm-mt-pool) |
[GitHub](https://github.com/w3reality/wasm-mt/tree/master/crates/pool) |
[Crate](https://crates.io/crates/wasm-mt-pool)

A thread pool library based on [`wasm-mt`](https://crates.io/crates/wasm-mt).

<!--
**Examples**:

You can run all the following examples in browser!

- **pool_exec** - How to use the lib. [ live | source ]
- **http** - How to use the lib. [ live | source ]
-->

# Getting started

Requirements:

- rustc (nightly)
- [`wasm-pack build`](https://github.com/rustwasm/wasm-pack#%EF%B8%8F-commands) with the [`--target no-modules`](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#without-a-bundler) option

Cargo.toml:

```
wasm-mt-pool = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_closure = "0.2"
```

# Usage

```
#![feature(async_closure)]

use wasm_mt_pool::prelude::*;
use wasm_mt::utils::{console_ln, sleep};

let size = 2;
let pkg_js = "./pkg/pool_exec.js"; // path to `wasm-bindgen`'s JS binding
let pool = ThreadPool::new(size, pkg_js).and_init().await.unwrap();

let num = 4;

console_ln!("a) 🔥 pool_exec! {} closures:", num);
for _ in 0..num {
    pool_exec!(pool, move || {
        console_ln!("a) closure: done.");
        Ok(JsValue::NULL)
    });
}

console_ln!("b) 🔥 pool_exec! {} async closures:", num);
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

console_ln!("c) 🔥 pool_exec! {} closures with callback:", num);
for _ in 0..num {
    pool_exec!(pool, move || {
        console_ln!("c) closure: done.");
        Ok(JsValue::from("C"))
    }, cb);
}

console_ln!("d) 🔥 pool_exec! {} async closures with callback:", num);
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
