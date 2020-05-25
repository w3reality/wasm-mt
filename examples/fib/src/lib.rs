#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use futures_channel::oneshot;
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, run_js, u8arr_from_vec};

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        // Full path required due to recursive instantiation of threads
        let mut href = run_js("return location.href;").unwrap().as_string().unwrap();
        let pkg_js_uri = if href.contains("index.html") {
            href.replace("index.html", "pkg/fib.js")
        } else {
            href.push_str("/pkg/fib.js");
            href
        };

        let _ = run(&pkg_js_uri, None).await;
    });
}

type ResultJJ = Result<JsValue, JsValue>;

pub async fn run(pkg_js_uri: &str, ab_init_test: Option<ArrayBuffer>) -> ResultJJ {
/*
num:   0 1 2 3 4 5 6 ...
(fib): 0 1 1 2 3 5 8 ...

5 (5)
├── 3 (2)
│   ├── 1
│   └── 2 (1)
│       ├── 0
│       └── 1
└── 4 (3)
    ├── 2 (1)
    │   ├── 0
    │   └── 1
    └── 3 (2)
        ├── 1
        └── 2 (1)
            ├── 0
            └── 1
*/
    // let num = 3;
    let num = 4;
    // let num = 5;

    let ans = fib_mt(num, pkg_js_uri, ab_init_test).await?;
    console_ln!("num: {}, fib_mt: ans: {}", num, ans);
    assert_eq!(ans, 3);

    Ok(JsValue::from(0))
}

// Note: On Safari, nested Web Workers might not be supported as of now.
async fn fib_mt(num: u32, pkg_js_uri: &str, ab_init_test: Option<ArrayBuffer>) -> Result<u32, JsValue> {
    if num <= 1 {
        console_ln!("fib({}): returns {}", num, num);
        return Ok(num);
    }

    let mt = WasmMt::new(pkg_js_uri).and_init().await?;

    let mut vec_init_test = None;
    if let Some(ab) = ab_init_test {
        vec_init_test = Some(Uint8Array::new(&ab).to_vec());
        mt.set_ab_init(ab);
    }

    let th = mt.thread().and_init().await?;

    console_ln!("fib({}): spawns fib({}) and fib({})", num, num - 1, num - 2);
    let pkg_js_uri = String::from(pkg_js_uri);

    let ans = exec!(th, async move || -> ResultJJ {
        let mut ab_left = None;
        let mut ab_right = None;
        if let Some(vec) = vec_init_test {
            ab_left = Some(u8arr_from_vec(&vec).buffer());
            ab_right = Some(u8arr_from_vec(&vec).buffer());
        }

        let (tx, rx) = oneshot::channel::<u32>();
        spawn_local(async move {
            let left = fib_mt(num - 2, &pkg_js_uri, ab_left).await.unwrap();
            let right = fib_mt(num - 1, &pkg_js_uri, ab_right).await.unwrap();
            tx.send(left + right).unwrap();
        });
        let ans_inner = rx.await.unwrap();
        console_ln!("fib({}): returns {}", num, ans_inner);
        Ok(JsValue::from(ans_inner))
    }).await.unwrap();

    Ok(ans.as_f64().unwrap() as u32)
}
