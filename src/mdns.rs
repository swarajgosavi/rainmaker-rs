pub(crate) mod base;
pub use base::*;

#[cfg(target_os="espidf")]
pub mod mdns_esp;

// #[cfg(target_os="espidf")]
// pub type MdnsService = base::MdnsService<esp_idf_svc::mdns::EspMdns>;

#[cfg(target_os="linux")]
pub mod mdns_linux;

// #[cfg(target_os="linux")]
// pub type MdnsService = base::MdnsService<LinuxMdns>;