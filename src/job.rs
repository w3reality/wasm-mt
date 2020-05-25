use std::future::Future;
use std::rc::Rc;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use js_sys::{ArrayBuffer, Uint8Array};
use serde::{de::DeserializeOwned, Serialize};
use serde_closure::FnOnce;
use serde_traitobject;
use super::utils;
use super::atw::{ThreadWorker as AtwThreadWorker};

type ResultJJ = Result<JsValue, JsValue>;

pub trait MtClosure = FnOnce() -> ResultJJ + Serialize + DeserializeOwned + 'static;
pub trait MtAsyncClosure<T> = FnOnce() -> T + Serialize + DeserializeOwned + 'static
    where T: Future<Output = ResultJJ> + 'static;

fn send_result(result: ResultJJ, atw_thw: Rc<AtwThreadWorker>, req_id: &str) {
    match result {
        // TODO !!!! optimise transferables cases
        Ok(ref ret) => atw_thw.send_response(req_id, ret, None),
        Err(ref ret) => atw_thw.send_error(req_id, ret),
    }
}

pub fn run_job_js(jsv: &JsValue, atw_thw: Rc<AtwThreadWorker>, req_id: &str, is_async: bool) {
    let js = jsv.as_string().unwrap();
    if is_async {
        let req_id = req_id.to_string();
        spawn_local(async move {
            send_result(utils::run_js_async(js.as_str()).await, atw_thw, &req_id);
        });
    } else {
        send_result(utils::run_js(js.as_str()), atw_thw, req_id);
    }
}

pub struct Job<T> {
    clos_fold: Box<dyn serde_traitobject::FnOnce<(Rc<AtwThreadWorker>, String,), Output = ()> + 'static>,
    _phantom: PhantomData<T>,
}

impl<T> Job<T> where T: Future<Output = ResultJJ> + 'static {
    pub fn from_clos<F>(clos: F) -> ArrayBuffer where F: MtClosure {
        let vec: Vec<u8> = bincode::serialize(&clos).unwrap();
        { #[allow(warnings)] {
            (Self {
                clos_fold: Box::new(FnOnce!(move |atw_thw: Rc<AtwThreadWorker>, req_id: String| {
                    let clos: F = bincode::deserialize(&vec).unwrap();
                    send_result(clos(), atw_thw, &req_id);
                })),
                _phantom: PhantomData,
            }).to_ab()
        } }
    }
    pub fn from_aclos<F>(clos: F) -> ArrayBuffer where F: MtAsyncClosure<T> {
        let vec: Vec<u8> = bincode::serialize(&clos).unwrap();
        { #[allow(warnings)] {
            (Self {
                clos_fold: Box::new(FnOnce!(move |atw_thw: Rc<AtwThreadWorker>, req_id: String| {
                    spawn_local(async move {
                        let clos: F = bincode::deserialize(&vec).unwrap();
                        send_result(clos().await, atw_thw, &req_id);
                    });
                })),
                _phantom: PhantomData,
            }).to_ab()
        } }
    }
    pub fn run(jsv: &JsValue, atw_thw: Rc<AtwThreadWorker>, req_id: &str) {
        let ab = jsv.dyn_ref::<ArrayBuffer>().unwrap();
        (Self::from_ab(ab).clos_fold)(atw_thw, String::from(req_id));
    }
    fn to_ab(&self) -> ArrayBuffer {
        let vec = bincode::serialize(&self.clos_fold).unwrap();
        utils::u8arr_from_vec(&vec).buffer()
    }
    fn from_ab(ab: &JsValue) -> Self {
        let vec: Vec<u8> = Uint8Array::new(ab).to_vec();
        Self {
            clos_fold: bincode::deserialize(&vec).unwrap(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;

    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn closures() {
        let ok42 = Ok(JsValue::from(42));
        assert_eq!(
            exec_fn(FnOnce!(move || Ok(JsValue::from(42)) )), ok42);
        assert_eq!(
            exec_aclos(FnOnce!(async move || Ok(JsValue::from(42)) )).await, ok42);
        assert_eq!(
            exec_pin(FnOnce!(move || Box::pin( afn_foo(42) ))).await, ok42);
        assert_eq!(
            exec_pin(FnOnce!(move || Box::pin( (async move || Ok(JsValue::from(42)))() ))).await, ok42);
    }

    async fn afn_foo(s: u32) -> ResultJJ { Ok(JsValue::from(s)) }

    fn exec_fn<F>(clos: F) -> ResultJJ where F: FnOnce() -> ResultJJ + Serialize + DeserializeOwned {
        let vec = bincode::serialize(&clos).unwrap();
        let recovered: F = bincode::deserialize(&vec).unwrap();
        recovered()
    }

    async fn exec_aclos<F, G>(clos: F) -> ResultJJ where F: FnOnce() -> G + Serialize + DeserializeOwned, G: Future<Output = ResultJJ> {
        let vec = bincode::serialize(&clos).unwrap();
        let recovered: F = bincode::deserialize(&vec).unwrap();
        recovered().await
    }

    // https://users.rust-lang.org/t/receive-an-async-function-as-a-parameter/33955/2
    async fn exec_pin<F, G>(clos: F) -> ResultJJ where F: FnOnce() -> Pin<Box<G>> + Serialize + DeserializeOwned, G: Future<Output = ResultJJ> {
        let vec = bincode::serialize(&clos).unwrap();
        let recovered: F = bincode::deserialize(&vec).unwrap();
        recovered().await
    }
}
