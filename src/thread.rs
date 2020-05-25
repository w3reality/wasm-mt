use crate::debug_ln;
use std::rc::Rc;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::prelude::*;
use js_sys::{Array, ArrayBuffer, Object, Reflect};
use web_sys::{Blob, BlobPropertyBag, Url};
use super::atw::Thread as AtwThread;
use super::job;
use super::encode_task_msg;

type ResultJJ = Result<JsValue, JsValue>;

pub struct Thread {
    ab_init: RefCell<Option<ArrayBuffer>>,
    ab_wasm: RefCell<Option<ArrayBuffer>>,
    atw_th: AtwThread,
    is_initialized: RefCell<bool>,
    id: RefCell<Option<Rc<String>>>,
    is_busy: RefCell<bool>,
}

impl Thread {
    fn create_blob_url(content: &str) -> String {
        // rust-wasm equivalent of --
        //   return URL.createObjectURL(
        //     new Blob([content], {type: 'text/javascript'}));
        let blob = Blob::new_with_str_sequence_and_options(
                &Array::of1(&JsValue::from(content)),
                BlobPropertyBag::new().type_("text/javascript"))
            .unwrap();

        Url::create_object_url_with_blob(&blob).unwrap()
    }
    fn revoke_blob_url(blob_url: String) {
        Url::revoke_object_url(&blob_url).unwrap();
    }

    fn get_worker_content() -> &'static str {
        "
        const instantiate = async (abInit, abWasm) => {
            // console.log('abInit:', abInit);
            const initJs = new TextDecoder().decode(abInit);
            const init = (new Function(initJs)).call(null);
            const wbg = init();
            const wasm = await wbg(abWasm);
            // console.log('wbg:', wbg);
            // console.log('wasm:', wasm);
            return { wbg, wasm };
        };

        let first = true;
        self.onmessage = async e => {
            // console.log('onmessage(): e.data', e.data);

            const { id, payload } = e.data; // destructure the initial `atw` msg
            const { abInit, abWasm } = payload;
            if (first) {
                first = false;
                try {
                    const { wbg, wasm } = await instantiate(abInit, abWasm);
                    // throw 'ok, bye for now'; // !! debug

                    // This overrides `self.onmessage`
                    const _worker = wbg.wmt_bootstrap(self, id);

                    self.wmtContext = { wbg, wasm, _worker };
                    // console.log('bootstrap complete - self.wmtContext:', self.wmtContext);
                } catch (e) {
                    console.log('bootstrap error:', e);
                }
                return;
            }
            throw 'oh no';
        };
        "
    }

    pub fn new(ab_init: ArrayBuffer, ab_wasm: ArrayBuffer) -> Self {
        let blob_url = Self::create_blob_url(Self::get_worker_content());
        debug_ln!("blob_url: {}", &blob_url);
        let atw_th = AtwThread::new(&blob_url);
        Self::revoke_blob_url(blob_url);

        Self {
            ab_init: RefCell::new(Some(ab_init)),
            ab_wasm: RefCell::new(Some(ab_wasm)),
            atw_th,
            is_initialized: RefCell::new(false),
            id: RefCell::new(None),
            is_busy: RefCell::new(false),
        }
    }

    pub async fn init(&self) -> Result<&Self, JsValue> {
        let ab_init = self.ab_init.replace(None).unwrap_throw();
        let ab_wasm = self.ab_wasm.replace(None).unwrap_throw();

        let payload = Object::new();
        Reflect::set(payload.as_ref(), &JsValue::from("abInit"), &ab_init).unwrap();
        Reflect::set(payload.as_ref(), &JsValue::from("abWasm"), &ab_wasm).unwrap();

        let result = self.atw_th.send_request(
            &payload, Some(&Array::of2(&ab_init, &ab_wasm))).await;
        let result = match result {
            Ok(jsv) => format!("ok: {}", jsv.as_string().unwrap()),
            Err(jsv) => format!("err: {}", jsv.as_string().unwrap()),
        };
        debug_ln!("init() - result: {}", result);

        self.is_initialized.replace(true);

        Ok(self)
    }

    pub async fn and_init(self) -> Result<Self, JsValue> {
        self.init().await?;
        Ok(self)
    }

    pub async fn exec<F>(&self, clos: F) -> ResultJJ where F: job::MtClosure {
        assert!(*self.is_initialized.borrow());

        type _TypeT = Pin<Box<dyn Future<Output = ResultJJ>>>;
        let ab = job::Job::<_TypeT>::from_clos(clos);
        let msg = encode_task_msg("job-clos", Some(&ab));
        self.atw_th.send_request(&msg, Some(&Array::of1(&ab))).await
    }

    pub async fn exec_async<F, T>(&self, aclos: F) -> ResultJJ where F: job::MtAsyncClosure<T> {
        assert!(*self.is_initialized.borrow());

        let ab = job::Job::<T>::from_aclos(aclos);
        let msg = encode_task_msg("job-aclos", Some(&ab));
        self.atw_th.send_request(&msg, Some(&Array::of1(&ab))).await
    }

    pub async fn exec_js(&self, js: &str) -> ResultJJ {
        let msg = encode_task_msg("job-js", Some(&JsValue::from(js)));
        self.atw_th.send_request(&msg, None).await
    }

    pub async fn exec_js_async(&self, js: &str) -> ResultJJ {
        let msg = encode_task_msg("job-js-async", Some(&JsValue::from(js)));
        self.atw_th.send_request(&msg, None).await
    }

    pub fn terminate(&self) {
        self.atw_th.terminate();
    }

    pub fn get_id(&self) -> Option<Rc<String>> {
        self.id.borrow().as_ref().cloned()
    }

    pub fn set_id(&self, id: &str) -> &Self {
        self.id.replace(Some(Rc::new(String::from(id))));
        self
    }

    pub fn get_busy(&self) -> bool {
        *self.is_busy.borrow()
    }

    pub fn set_busy(&self, tf: bool) -> &Self {
        self.is_busy.replace(tf);
        self
    }
}
