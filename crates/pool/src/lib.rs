//! A thread pool library based on `wasm-mt` ([github](https://github.com/w3reality/wasm-mt) | [crate](https://crates.io/crates/wasm-mt)).
//!
//! #### Examples
//!
//! You can run all the following apps in browser!
//!
//! - **pool_exec** - How to use <code>wasm_mt_pool</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/pool_exec/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/pool_exec) ]
//! - **http** - A multithreaded server based on <code>wasm_mt_pool</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/http/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/http) ]
//! - **pool_arraybuffers** - Demo of using <code>ThreadPool::new_with_arraybuffers()</code>. [ [live](https://w3reality.github.io/wasm-mt/crates/pool/examples/pool_arraybuffers/index.html) | [source](https://github.com/w3reality/wasm-mt/tree/master/crates/pool/examples/pool_arraybuffers) ]
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
//! ```toml
//! wasm-mt-pool = "0.1"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_closure = "0.3"
//! ```
//!
//! # Usage
//!
//! ```rust
//! #![feature(async_closure)]
//!
//! use wasm_mt_pool::prelude::*;
//! use wasm_mt::utils::{console_ln, sleep};
//!
//! let size = 2;
//! let pkg_js = "./pkg/pool_exec.js"; // path to `wasm-bindgen`'s JS binding
//! let pool = ThreadPool::new(size, pkg_js).and_init().await.unwrap();
//!
//! let num = 4;
//!
//! console_ln!("a) 💦 pool_exec! {} closures:", num);
//! for _ in 0..num {
//!     pool_exec!(pool, move || {
//!         console_ln!("a) closure: done.");
//!         Ok(JsValue::NULL)
//!     });
//! }
//!
//! console_ln!("b) 💦 pool_exec! {} async closures:", num);
//! for _ in 0..num {
//!     pool_exec!(pool, async move || {
//!         sleep(1000).await;
//!         console_ln!("b) async closure: done.");
//!         Ok(JsValue::NULL)
//!     });
//! }
//!
//! let cb = move |result| {
//!     console_ln!("callback: result: {:?}", result);
//! };
//!
//! console_ln!("c) 💦 pool_exec! {} closures with callback:", num);
//! for _ in 0..num {
//!     pool_exec!(pool, move || {
//!         console_ln!("c) closure: done.");
//!         Ok(JsValue::from("C"))
//!     }, cb);
//! }
//!
//! console_ln!("d) 💦 pool_exec! {} async closures with callback:", num);
//! for _ in 0..num {
//!     pool_exec!(pool, async move || {
//!         sleep(1000).await;
//!         console_ln!("d) async closure: done.");
//!         Ok(JsValue::from("D"))
//!     }, cb);
//! }
//!
//! sleep(6_000).await; // Do sleep long enough to ensure all jobs are completed.
//! assert_eq!(pool.count_pending_jobs(), 0);
//! ```

#![feature(trait_alias)]
#![feature(async_closure)]

pub use wasm_mt;
use wasm_mt::{debug_ln, WasmMt, Thread, MtClosure, MtAsyncClosure};
use js_sys::ArrayBuffer;

pub mod prelude;
mod resolver;
use resolver::Resolver;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;

type ResultJJ = Result<JsValue, JsValue>;

pub trait PoolCallback = FnOnce(ResultJJ,) -> () + 'static;

struct ThreadPoolInner {
    size: usize,
    mt: WasmMt,
    threads: RefCell<Vec<Thread>>,
    resolver: Resolver,
}

impl ThreadPoolInner {
    fn new(size: usize, pkg_js_uri: &str) -> Self {
        assert!(size > 0);
        Self {
            size,
            mt: WasmMt::new(pkg_js_uri),
            threads: RefCell::new(Vec::with_capacity(size)),
            resolver: Resolver::new(),
        }
    }

    fn new_with_arraybuffers(size: usize, ab_js: ArrayBuffer, ab_wasm: ArrayBuffer) -> Self {
        assert!(size > 0);
        Self {
            size,
            mt: WasmMt::new_with_arraybuffers(ab_js, ab_wasm),
            threads: RefCell::new(Vec::with_capacity(size)),
            resolver: Resolver::new(),
        }
    }

    async fn init(&self) -> Result<(), JsValue> {
        let mut threads = self.threads.borrow_mut();

        self.mt.init().await?;
        for id in 0..self.size {
            let pth = self.mt.thread();
            pth.set_id(&id.to_string());
            threads.push(pth);
        }

        for pth in threads.iter() {
            pth.init().await?;
        }

        Ok(())
    }

    async fn execute<F>(&self, clos: F) -> ResultJJ where F: MtClosure {
        let threads = self.threads.borrow();
        let pth = self.resolver.resolve_runnable(&threads).await?;

        let result = pth.exec(clos).await;
        debug_ln!("pth {} done with result: {:?}", pth.get_id().unwrap(), result);
        self.resolver.notify_job_complete(pth);
        result
    }

    async fn execute_async<F, T>(&self, aclos: F) -> ResultJJ where F: MtAsyncClosure<T> {
        let threads = self.threads.borrow();
        let pth = self.resolver.resolve_runnable(&threads).await?;

        let result = pth.exec_async(aclos).await;
        debug_ln!("pth {} done with result: {:?}", pth.get_id().unwrap(), result);
        self.resolver.notify_job_complete(pth);
        result
    }

    async fn execute_js(&self, js: &str, is_async: bool) -> ResultJJ {
        let threads = self.threads.borrow();
        let pth = self.resolver.resolve_runnable(&threads).await?;

        let result = if is_async {
            pth.exec_js_async(js).await
        } else {
            pth.exec_js(js).await
        };
        debug_ln!("pth {} done with result: {:?}", pth.get_id().unwrap(), result);
        self.resolver.notify_job_complete(pth);
        result
    }

    fn drop_inner(&self) {
        debug_ln!("[drop] drop_inner(): terminating {} workers ...", self.size);
        self.resolver.cancel_pending_jobs();
        self.threads.borrow().iter().for_each(|pth| pth.terminate());
    }
}

#[macro_export]
macro_rules! pool_exec {
    ($pool:expr, async $clos:expr) => (($pool).exec_async(FnOnce!(async $clos)));
    ($pool:expr, $clos:expr) => (($pool).exec(FnOnce!($clos)));
    ($pool:expr, async $clos:expr, $cb:expr) => (($pool).exec_async_with_cb(FnOnce!(async $clos), $cb));
    ($pool:expr, $clos:expr, $cb:expr) => (($pool).exec_with_cb(FnOnce!($clos), $cb));
}

#[macro_export]
macro_rules! pool_exec_js {
    ($pool:expr, $str:expr) => (($pool).exec_js($str));
    ($pool:expr, $str:expr, $cb:expr) => (($pool).exec_js_with_cb($str, $cb));
}

#[macro_export]
macro_rules! pool_exec_js_async {
    ($pool:expr, $str:expr) => (($pool).exec_js_async($str));
    ($pool:expr, $str:expr, $cb:expr) => (($pool).exec_js_async_with_cb($str, $cb));
}


pub struct ThreadPool(Rc<ThreadPoolInner>);

impl Drop for ThreadPool {
    fn drop(&mut self) {
        debug_ln!("[drop] ThreadPool::drop(): sc: {}", Rc::strong_count(&self.0));
        self.0.drop_inner();
    }
}

impl ThreadPool {
    pub fn new(size: usize, pkg_js_uri: &str) -> Self {
        Self(Rc::new(ThreadPoolInner::new(size, pkg_js_uri)))
    }

    pub fn new_with_arraybuffers(size: usize, ab_js: ArrayBuffer, ab_wasm: ArrayBuffer) -> Self {
        Self(Rc::new(ThreadPoolInner::new_with_arraybuffers(size, ab_js, ab_wasm)))
    }

    pub fn set_ab_init(&self, ab: ArrayBuffer) {
        self.0.mt.set_ab_init(ab);
    }

    pub async fn init(&self) -> Result<&Self, JsValue> {
        self.0.init().await?;
        Ok(self)
    }

    pub async fn and_init(self) -> Result<Self, JsValue> {
        self.init().await?;
        Ok(self)
    }

    pub fn count_pending_jobs(&self) -> usize {
        self.0.resolver.count_pending_jobs()
    }

    fn drop_cb_result(_: ResultJJ) {}

    pub fn exec<F>(&self, job: F) where F: MtClosure {
        self.exec_with_cb(job, Self::drop_cb_result);
    }
    pub fn exec_async<F, T>(&self, job: F) where F: MtAsyncClosure<T> {
        self.exec_async_with_cb(job, Self::drop_cb_result);
    }
    pub fn exec_with_cb<F, G>(&self, job: F, cb: G) where
    F: MtClosure, G: PoolCallback {
        let pool_inner = self.0.clone();
        spawn_local(async move {
            cb(pool_inner.execute(job).await);
        });
    }
    pub fn exec_async_with_cb<F, T, G>(&self, job: F, cb: G) where
    F: MtAsyncClosure<T>, G: PoolCallback {
        let pool_inner = self.0.clone();
        spawn_local(async move {
            cb(pool_inner.execute_async(job).await);
        });
    }

    pub fn exec_js(&self, js: &str) {
        self.exec_js_inner(js, false, Self::drop_cb_result);
    }
    pub fn exec_js_async(&self, js: &str) {
        self.exec_js_inner(js, true, Self::drop_cb_result);
    }
    pub fn exec_js_with_cb<G>(&self, js: &str, cb: G) where G: PoolCallback {
        self.exec_js_inner(js, false, cb);
    }
    pub fn exec_js_async_with_cb<G>(&self, js: &str, cb: G) where G: PoolCallback {
        self.exec_js_inner(js, true, cb);
    }
    fn exec_js_inner<G>(&self, js: &str, is_async: bool, cb: G) where G: PoolCallback {
        let pool_inner = self.0.clone();
        let js = js.to_string();
        spawn_local(async move {
            cb(pool_inner.execute_js(js.as_str(), is_async).await);
        });
    }
}
