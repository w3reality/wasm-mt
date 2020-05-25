//! Utility for testing crates with [`wasm-mt-pool`](https://crates.io/crates/wasm-mt-pool).

use wasm_mt::console_ln;
use wasm_mt_pool::ThreadPool;
use wasm_mt_test::{create_ab_init, get_pkg_js_uri};

pub async fn create_pool(size: usize) -> ThreadPool {
    let pkg_js_uri = get_pkg_js_uri();

    let pool = ThreadPool::new(size, &pkg_js_uri);
    let ab = create_ab_init(&pkg_js_uri).await.unwrap();
    pool.set_ab_init(ab);

    pool.init().await.unwrap();
    console_ln!("pool is ready now!");

    pool
}
