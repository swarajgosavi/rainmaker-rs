pub const MDNS_PORT: u16 = 5353;

pub struct MdnsService<T> {
    #[allow(unused)]
    pub(crate) mdns_service: T,
}