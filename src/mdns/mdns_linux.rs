use super::MdnsService;

use zeroconf::prelude::*;
use zeroconf::ServiceType;

pub struct LinuxMdns (());

impl LinuxMdns {
    pub fn take() -> anyhow::Result<Self> {

        log::info!("Initializing MDNS");

        Ok(Self(()))
    }
}

impl MdnsService<LinuxMdns> {
    pub fn mdns_init() -> anyhow::Result<Self> {
        log::info!("mdns initialized");
        Ok(MdnsService{
            mdns_service: LinuxMdns::take().unwrap()
        })
    }

    pub fn mdns_hostname_set(&mut self, hostname: &str) {
        log::info!("Set hostname {}", hostname);
    }

    pub fn mdns_service_add(&mut self, instance_name: &str, service_type: &str, proto: &str, txt: &[(&str, &str)]) {


        
        let service_type = ServiceType::new(service_type, proto)
        .map_err(|err| {
            log::error!(
                "Encountered error building service type: {}",
                err.to_string()
            );
        }).unwrap();

        let mut txt_record = zeroconf::TxtRecord::new();
        for txt_val in txt {
            log::info!("mDNS TXT key {} val {}", txt_val.0, txt_val.1);
            if let Err(err) = txt_record.insert(txt_val.0, txt_val.1) {
                log::error!(
                    "Encountered error inserting txt-pair into txt record {}",
                    err.to_string()
                );
            }
        }

        let servicename = instance_name.to_owned();

        std::thread::spawn(move || {
            let mut mdns_service = zeroconf::MdnsService::new(service_type, 8080);
            mdns_service.set_name(&servicename);
            mdns_service.set_txt_record(txt_record);
            // mdns_service.set_registered_callback(Box::new(|_, _| {}));

            let context: Arc<Mutex<Context>> = Arc::default();

            mdns_service.set_registered_callback(Box::new(on_service_registered));
            mdns_service.set_context(Box::new(context));

            match mdns_service.register() {
                Ok(event_loop) => loop {
                    // break
                    if let Err(err) = event_loop.poll(std::time::Duration::from_secs(1)) {
                        log::error!(
                            "Failed to poll mDNS service event loop: {}",
                            err.to_string()
                        );
                        break;
                    }
                    // log::info!("in loop");
                },
                Err(err) => log::error!(
                    "Encountered error registering mDNS service: {}",
                    err.to_string()
                ),
            }
        });
    }
}


use std::any::Any;
use std::sync::{Arc, Mutex};
use zeroconf::ServiceRegistration;

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn on_service_registered(
    result: zeroconf::Result<ServiceRegistration>,
    context: Option<Arc<dyn Any>>,
) {
    let service = result.expect("failed to register service");

    log::info!("Service registered: {:?}", service);

    let context = context
        .as_ref()
        .expect("could not get context")
        .downcast_ref::<Arc<Mutex<Context>>>()
        .expect("error down-casting context")
        .clone();

    context
        .lock()
        .expect("failed to obtain context lock")
        .service_name = service.name().clone();

    log::info!("Context: {:?}", context);

    // ...
}