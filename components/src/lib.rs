#![feature(map_try_insert)] // used in http_linux.rs
#![feature(trait_alias)] // http_linux.rs

pub mod error;
pub mod http;
pub mod mqtt;
pub mod persistent_storage;
pub mod protocomm;
pub mod wifi;

pub(crate) mod utils;

fn hello(name: &str) {
    log::info!("hello from {}", name);
}

pub fn say_hello() {
    #[cfg(target_os = "espidf")]
    hello("espidf");

    #[cfg(target_os = "linux")]
    hello("linux");
}
