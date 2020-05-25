use wasm_mt::{debug_ln, Thread};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Promise, Function};
use std::collections::VecDeque;
use std::cell::RefCell;

pub struct Resolver {
    queue: RefCell<VecDeque<(Function, Function)>>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub async fn resolve_runnable<'a>(&self, threads: &'a Vec<Thread>) -> Result<&'a Thread, JsValue> {
        for pth in threads.iter() {
            if !pth.get_busy() {
                debug_ln!("[resolver] immediate resolution to pth: {}", pth.get_id().unwrap());
                return Ok(pth.set_busy(true));
            }
        }

        debug_ln!("[resolver] deferring resolution... pth: ?");
        let promise = Promise::new(&mut |res, rej| self.queue.borrow_mut().push_back((res, rej)));

        match JsFuture::from(promise).await {
            Ok(ref jsv) => {
                let id = jsv.as_string().unwrap()
                    .parse::<usize>().unwrap();

                debug_ln!("[resolver] a queued promise resolved to id: {}", id);

                let pth = &threads[id];
                if !pth.get_busy() {
                    let msg = "[resolver] the resolved thread is in illegal state";
                    debug_ln!("calling `panic!()`: {}", msg);
                    panic!(msg);
                }

                Ok(pth)
            },
            Err(jsv) => {
                debug_ln!("[resolver] a queued promise rejected with: {:?}", jsv);

                Err(jsv)
            },
        }
    }

    pub fn notify_job_complete(&self, pth: &Thread) {
        if let Some((res, _rej)) = self.queue.borrow_mut().pop_front() {
            // let the pending `resolve_runnable()` return for one more round to go
            res.call1(&JsValue::NULL, &JsValue::from(pth.get_id().unwrap().as_ref())).unwrap();
        } else {
            pth.set_busy(false);
        }
    }

    pub fn cancel_pending_jobs(&self) {
        let mut queue = self.queue.borrow_mut();

        let cancels = queue.len();
        debug_ln!("cancel_pending_jobs(): canceling {} pending jobs", cancels);
        let mut count = 0;
        while let Some((_res, rej)) = queue.pop_front() {
            rej.call1(&JsValue::NULL,
                &JsValue::from(format!("ThreadPool: job[{}] canceled", count))).unwrap();
            count += 1;
        }
    }

    pub fn count_pending_jobs(&self) -> usize {
        self.queue.borrow().len()
    }
}
