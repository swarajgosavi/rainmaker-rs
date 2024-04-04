use super::MdnsService;

impl MdnsService<esp_idf_svc::mdns::EspMdns> {
    pub fn mdns_init() -> anyhow::Result<Self> {
        Ok(MdnsService { 
            mdns_service: esp_idf_svc::mdns::EspMdns::take().unwrap() 
        })
    }

    pub fn mdns_hostname_set(&mut self, hostname: &str) {
        self.mdns_service
            .set_hostname(hostname)
            .unwrap();
    }

    pub fn mdns_service_add(&mut self, instance_name: &str, service_type: &str, proto: &str, txt: &[(&str, &str)]) {

        let service_type = format!("_{}", service_type);
        let proto = format!("_{}", proto);

        self.mdns_service
            .add_service(Some(instance_name), &service_type, &proto, 8080, txt)
            .unwrap();
    }
}