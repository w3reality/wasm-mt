#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use wasm_mt::utils::{console_ln, debug_ln, sleep};
use wasm_mt_pool::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use rand::Rng;

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let size = 2;
        let pool = ThreadPool::new(size, "./pkg/http.js").and_init().await.unwrap();
        console_ln!("a work-stealing thread pool (with {} threads) is ready now!", size);

        let _ = run(pool).await;
    });
}

type ResultJJ = Result<JsValue, JsValue>;

pub async fn run(pool: ThreadPool) -> ResultJJ {
    let num_pages = 7;
    let ms_delay_page = 1000;

    let log = Rc::new(HttpLog::new(num_pages));
    for i in 0..num_pages {
        let log = log.clone();
        let cb = move |result: ResultJJ| {
            debug_ln!("callback: result: {:?}", result);
            if let Ok(ref jsv) = result {
                log.append_contents(jsv.as_string().unwrap().as_str());
            }
        };

        console_ln!("client: requesting page-{}", i);
        pool_exec!(pool, async move || -> ResultJJ {
            let page = format!("  page-{}-content-{}", i, rand::thread_rng().gen_range(0.0, 1.0));
            sleep(ms_delay_page).await;
            Ok(JsValue::from(&page))
        }, cb);
    }

    if false {
        // Test 'canceled' cases.  Try building with `--dev` to see more details.
        sleep(2000).await; // `pool` drops leaving some requests!!
    } else {
        sleep(5000).await;
        assert_eq!(pool.count_pending_jobs(), 0);
    }

    console_ln!("bye.");
    Ok(JsValue::from(0))
}

struct HttpLog {
    num_pages: usize,
    num_pages_fetched: RefCell<usize>,
    log: RefCell<String>,
}

impl HttpLog {
    fn new(num_pages: usize) -> Self {
        Self {
            num_pages,
            num_pages_fetched: RefCell::new(0),
            log: RefCell::new(String::new()),
        }
    }
    fn append_contents(&self, contents: &str) {
        let mut fetched = self.num_pages_fetched.borrow_mut();
        *fetched += 1;
        console_ln!("fetched/total: {}/{}", fetched, self.num_pages);

        self.log.borrow_mut().push_str(contents);
        console_ln!("log: {}", &self.log.borrow());
    }
}
