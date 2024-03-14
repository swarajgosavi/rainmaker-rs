use crate::protocomm::{protocomm_req_handler, CallbackData};
use crate::protocomm::transports::TransportTrait;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use crate::http::{HttpConfiguration, HttpMethod, HttpRequest, HttpResponse, HttpServer};

use crate::utils::WrappedInArcMutex;
use super::TransportCallbackType;

pub(crate) struct TransportHttpd<'a> {
    http_server: WrappedInArcMutex<HttpServer<'a>>,
    cb_data: Option<WrappedInArcMutex<CallbackData>>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> TransportHttpd<'a> {
    pub fn new(config: HttpConfiguration) -> Self {
        let http_server = HttpServer::new(&config).unwrap();
        Self {
            http_server: Arc::new(Mutex::new(http_server)),
            phantom: PhantomData::default(),
            cb_data: None
        }
    }

    pub(crate) fn register_cb_data(&mut self, data: Arc<Mutex<CallbackData>>){
        self.cb_data = Some(data)
    }
}

impl<'a> TransportTrait for TransportHttpd<'a> {
    fn add_endpoint(&self, ep_name: &str, cb: impl TransportCallbackType) {
        let mut http_server = self.http_server.lock().unwrap();
        let ep = "/".to_string() + &ep_name;
        // doing this works for small number of arguments
        // but cloning it for every endpoint isn't most efficient solution
        let cb_data = self.cb_data.clone().unwrap();
        http_server.add_listener(
            ep,
            HttpMethod::POST,
            Box::new(move |req| -> HttpResponse { http_callback(req, cb.borrow(), cb_data.to_owned()) }),
        );
    }
}

fn http_callback<T>(mut req: HttpRequest, cb: T, cb_data: Arc<Mutex<CallbackData>>) -> HttpResponse
where
    T: Fn(String, Vec<u8>) -> Vec<u8>,
{
    let url = req.url();
    let data = req.data();
    let ep = url.split_at(1).1.to_owned();

    let data_ret = protocomm_req_handler(ep, data, cb, cb_data);

    HttpResponse::from_bytes(data_ret)
}