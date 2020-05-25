//! A multithreading library for Rust and WebAssembly.
//!
//! `wasm-mt` helps you create and execute Web Worker based threads. You can program the threads simply using Rust closures and orchestrate them with `async/await`.
//!
//! <!--
//! **Implementation**:
//!
//! TODO: explain wasm-mt's approach to rustwasm multithreading in contrast to the [existing canonical work](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html "Parallelism through Web Workers")
//! -->
//!
//! <!--
//! **Examples**:
//!
//! You can run all the following examples in browser!
//!
//! - **exec** - How to use the lib. [ live | source ]
//! - **fib** - How to use the lib. [ live | source ]
//! - **executors** - How to use the lib. [ live | source ]
//! - **parallel** - How to use the lib. [ live | source ]
//! - **`wasm-mt-pool`** - Thread pool based on `wasm-mt`. [ crate | source ]
//! -->
//!
//! **Thanks**:
//!
//! - [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) developers
//! - [@alecmocatta](https://github.com/alecmocatta) for the [serde_traitobject](https://github.com/alecmocatta/serde_traitobject) crate
//!
//! # Getting started
//!
//! Requirements:
//!
//! - rustc (nightly)
//! - [`wasm-pack build`](https://github.com/rustwasm/wasm-pack#%EF%B8%8F-commands) with the [`--target no-modules`](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#without-a-bundler) option
//!
//! Cargo.toml:
//!
//! ```
//! wasm-mt = "0.1"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_closure = "0.2"
//! ```
//!
//! # Creating a thread
//!
//! First, create a [`WasmMt`] thread builder with [`new`][WasmMt::new] and initialize it:
//!
//! ```
//! use wasm_mt::prelude::*;
//!
//! let pkg_js = "./pkg/exec.js"; // path to `wasm-bindgen`'s JS binding
//! let mt = WasmMt::new(pkg_js).and_init().await.unwrap();
//! ```
//!
//! Then, create a [`wasm_mt::Thread`][Thread] with the [`thread`][WasmMt::thread] function and initialize it:
//!
//! ```
//! let th = mt.thread().and_init().await.unwrap();
//! ```
//!
//! # Executing a thread
//!
//! Using the [`exec!`] macro, you can execute a closure in the thread and `await` the result:
//!
//! ```
//! // fn add(a: i32, b: i32) -> i32 { a + b }
//!
//! let a = 1;
//! let b = 2;
//! let ans = exec!(th, move || {
//!     let c = add(a, b);
//!
//!     Ok(JsValue::from(c))
//! }).await?;
//! assert_eq!(ans, JsValue::from(3));
//! ```
//!
//! You can also execute an [async closure] with `exec!`:
//!
//! ```
//! // use wasm_mt::utils::sleep;
//! // async fn sub(a: i32, b: i32) -> i32 {
//! //    sleep(1000).await;
//! //    a - b
//! // }
//!
//! let a = 1;
//! let b = 2;
//! let ans = exec!(th, async move || {
//!     let c = sub(a, b).await;
//!
//!     Ok(JsValue::from(c))
//! }).await?;
//! assert_eq!(ans, JsValue::from(-1));
//! ```
//!
//! # Executing JavaScript in a thread
//!
//! Using the [`exec_js!`] macro, you can execute JavaScript within a thread:
//!
//! ```
//! let ans = exec_js!(th, "
//!     const add = (a, b) => a + b;
//!     return add(1, 2);
//! ").await?;
//! assert_eq!(ans, JsValue::from(3));
//! ```
//!
//! Similarly, use [`exec_js_async!`] for running asynchronous JavaScript:
//!
//! ```
//! let ans = exec_js_async!(th, "
//!     const sub = (a, b) => new Promise(resolve => {
//!         setTimeout(() => resolve(a - b), 1000);
//!     });
//!     return await sub(1, 2);
//! ").await?;
//! assert_eq!(ans, JsValue::from(-1));
//! ```
//!
//! # Making executors
//!
//! By using [`wasm_mt:Thread`][Thread], you can easily create custom executors. One such example is the [`wasm-mt-pool` crate](https://crates.io/crates/wasm-mt-pool). It provides a [thread pool](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#improving-throughput-with-a-thread-pool) that is based on the [work stealing] scheduling strategy.
//!
//! Here, for simplicity, we show the implementation of much more  straightforward executors: a serial executor and a parallel executor.
//!
//! First, prepare a `Vec<wasm_mt::Thread>` containing initialized threads:
//!
//! ```
//! let mut v: Vec<wasm_mt::Thread> = vec![];
//! for i in 0..4 {
//!     let th = mt.thread().and_init().await?;
//!     v.push(th);
//! }
//! ```
//!
//! Then, here's the executors in action. Note, in the latter case, we are using [`wasm_bindgen_futures::spawn_local`](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.spawn_local.html) to dispatch the threads in parallel.
//!
//! ```
//! console_ln!("🔥 serial executor:");
//! for th in &v {
//!     console_ln!("starting a thread");
//!     let ans = exec!(th, move || Ok(JsValue::from(42))).await?;
//!     console_ln!("ans: {:?}", ans);
//! }
//!
//! console_ln!("🔥 parallel executor:");
//! for th in v {
//!     spawn_local(async move {
//!         console_ln!("starting a thread");
//!         let ans = exec!(th, move || Ok(JsValue::from(42))).await.unwrap();
//!         console_ln!("ans: {:?}", ans);
//!     });
//! }
//! ```
//!
//! Observe the starting/ending timing of each thread in the developer console:
//!
//! ```
//! 🔥 serial executor:
//! starting a thread
//! ans: JsValue(42)
//! starting a thread
//! ans: JsValue(42)
//! starting a thread
//! ans: JsValue(42)
//! starting a thread
//! ans: JsValue(42)
//! 🔥 parallel executor:
//! (4) starting a thread
//! (4) ans: JsValue(42)
//! ```
//!
//! [async closure]: https://github.com/rust-lang/rfcs/blob/master/text/2394-async_await.md#async--closures
//! [work stealing]: https://en.wikipedia.org/wiki/Work_stealing

#![feature(trait_alias)]
#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use js_sys::{ArrayBuffer, Object, Reflect};
use web_sys::TextEncoder;
use std::cell::RefCell;

pub mod prelude;
pub mod utils;
mod job;
mod atw;
mod worker;
mod thread;

pub use job::{MtClosure, MtAsyncClosure};
pub use thread::Thread;

#[macro_export]
macro_rules! console_ln {
    ( $( $x:expr ),* ) => (web_sys::console::log_1(&format!( $( $x ),* ).into()));
}

#[macro_export]
macro_rules! debug_ln {
    ( $( $x:expr ),* ) => {
        if cfg!(debug_assertions) {
            let mut ln = String::from("👀 ");
            ln.push_str(&format!( $( $x ),* ));
            web_sys::console::log_1(&ln.into());
        }
    };
}

#[macro_export]
macro_rules! exec {
    ($th:expr, async $clos:expr) => (($th).exec_async(FnOnce!(async $clos)));
    ($th:expr, $clos:expr) => (($th).exec(FnOnce!($clos)));
}

#[macro_export]
macro_rules! exec_js { ($th:expr, $str:expr) => (($th).exec_js($str)); }

#[macro_export]
macro_rules! exec_js_async { ($th:expr, $str:expr) => (($th).exec_js_async($str)); }

pub struct WasmMt {
    pkg_js_uri: String,
    ab_init: RefCell<Option<ArrayBuffer>>,
    ab_wasm: RefCell<Option<ArrayBuffer>>,
}

impl WasmMt {
    pub fn new(pkg_js_uri: &str) -> Self {
        debug_ln!("pkg_js_uri: {}", pkg_js_uri);
        Self {
            pkg_js_uri: String::from(pkg_js_uri),
            ab_init: RefCell::new(None),
            ab_wasm: RefCell::new(None),
        }
    }

    pub fn set_ab_init(&self, ab: ArrayBuffer) {
        self.ab_init.replace(Some(ab));
    }

    pub async fn init(&self) -> Result<&Self, JsValue> {
        let pkg_wasm_uri = if self.pkg_js_uri.ends_with("wasm-bindgen-test") {
            // We defer updating `self.ab_init` in this 'test' context

            format!("{}_bg.wasm", &self.pkg_js_uri)
        } else {
            let ab_init = Self::create_ab_init(&self.pkg_js_uri).await?;
            self.set_ab_init(ab_init);

            self.pkg_js_uri.replace(".js", "_bg.wasm")
        };

        if !pkg_wasm_uri.ends_with("_bg.wasm") {
            wasm_bindgen::throw_str("failed to resolve `pkg_wasm_uri`");
        }
        let ab_wasm = utils::fetch_as_arraybuffer(&pkg_wasm_uri).await?;
        self.ab_wasm.replace(Some(ab_wasm));

        Ok(self)
    }

    pub async fn and_init(self) -> Result<Self, JsValue> {
        self.init().await?;
        Ok(self)
    }

    pub fn thread(&self) -> Thread {
        // https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.ArrayBuffer.html#method.slice
        Thread::new(
            self.ab_init.borrow().as_ref().unwrap().slice(0),
            self.ab_wasm.borrow().as_ref().unwrap().slice(0))
    }

    async fn create_ab_init(pkg_js_uri: &str) -> Result<ArrayBuffer, JsValue> {
        let pkg_js = utils::fetch_as_text(pkg_js_uri).await?;
        // debug_ln!("pkg_js: {}", &pkg_js);

        let mut init_js = String::new();
        init_js.push_str("return () => { ");
        init_js.push_str(&pkg_js);
        init_js.push_str(" return wasm_bindgen; };");
        // debug_ln!("init_js: {}", init_js);

        let ab_init = utils::u8arr_from_vec(
            &TextEncoder::new()?.encode_with_input(&init_js)).buffer();

        Ok(ab_init)
    }
}

fn encode_task_msg(name: &str, data: Option<&JsValue>) -> Object {
    let msg = Object::new();
    Reflect::set(msg.as_ref(), &JsValue::from("task"), &JsValue::from(name)).unwrap();
    if let Some(jsv) = data {
        Reflect::set(msg.as_ref(), &JsValue::from("data"), jsv).unwrap();
    }
    msg
}

fn decode_task_msg(msg: &JsValue) -> (String, JsValue) {
    let name = Reflect::get(msg, &JsValue::from("task"))
        .unwrap_throw().as_string().unwrap_throw();
    let jsv = Reflect::get(msg, &JsValue::from("data"))
        .unwrap_throw();
    (name, jsv)
}
