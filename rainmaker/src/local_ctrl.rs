use components::{http::HttpConfiguration, protocomm::*};
use crate::node::Node;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
};


const LOCAL_CTRL_VER: &str = "v1.1";
const LOGGER_TAH: &str = "local_ctrl";
const CAP_LOCAL_CTRL: &str = "wifi_scan"; // wifi scan capability
const CAP_NO_SEC: &str = "no_sec"; // capability signifying sec0
const CAP_NO_POP: &str = "no_pop"; // no PoP in case of sec1 and sec2

#[derive(Default)]
pub enum LocalCtrlScheme {
    #[default]
    SoftAP,
}

#[derive(Default)]
pub struct LocalCtrlConfig {
    pub device_name: String,
    pub scheme: LocalCtrlScheme,
    pub security: ProtocommSecurity,
}

pub struct LocalCtrlService<'a> {
    pub protocom: Protocomm<'a>,
    pub node: Arc<Node<'a>>,
}

impl<'a> LocalCtrlService<'a> 
where
    'a : 'static
{
    pub fn new(
        config: LocalCtrlConfig,
        node: Arc<Node<'a>>,
    ) -> Self {
        let version_info = Self::get_version_info(&config.security);
        let protocomm_config = ProtocommConfig {
            transport: ProtocomTransportConfig::Httpd(HttpConfiguration{
                port: 8080,
                ..Default::default()
            }),
            security: config.security,
        };

        let protocomm = Protocomm::new(protocomm_config);

        let mut local_ctrl_service = Self {
            protocom: protocomm,
            node
        };
        local_ctrl_service.init(version_info);

        local_ctrl_service
    }

    pub fn init(&mut self, version_info: serde_json::Value) {
        self.register_listeners(version_info);
    }

    pub fn register_listeners(&mut self, version_info: serde_json::Value) {
        log::debug!(target: LOGGER_TAH, "adding local_ctrl listeners");

        let node = self.node.clone();
        
        let pc = &mut self.protocom;
        pc.set_security_endpoint("esp_local_ctrl/session").unwrap();

        pc.set_version_endpoint("esp_local_ctrl/version", version_info.to_string())
            .unwrap();

        pc.register_endpoint("esp_local_ctrl/control", move |ep, data| -> Vec<u8> {
            control_handler(ep, data, node.to_owned())
        })
            .unwrap();
    }

    fn get_version_info(sec_config: &ProtocommSecurity) -> serde_json::Value {
        let mut local_capabilities = vec![CAP_LOCAL_CTRL];
        let sec_ver = match sec_config {
            ProtocommSecurity::Sec0(_) => {
                local_capabilities.push(CAP_NO_SEC);
                // return sec0
                0
            }
            ProtocommSecurity::Sec1(sec1_inner) => {
                if sec1_inner.pop.is_none() {
                    local_capabilities.push(CAP_NO_POP)
                };
                // return sec1
                1
            }
        };

        let ver_info = json!({
            "local_ctrl": {
                "ver": LOCAL_CTRL_VER,
                "sec_ver": sec_ver,
                "cap": local_capabilities
            }
        });

        ver_info
    }
}

pub fn control_handler(
    _ep: String,
    data: Vec<u8>,
    node: Arc<Node<'_>>
) -> Vec<u8> {

    let req_proto = LocalCtrlMessage::decode(&*data).unwrap();

    log::info!("local_ctrl_payload: {:?}", req_proto);

    match req_proto.payload.clone().unwrap() {
        local_ctrl_message::Payload::CmdGetPropCount(values) => {
            log::info!("values are {:?}", values);
        },
        local_ctrl_message::Payload::CmdGetPropVals(values) => {
            log::info!("values are {:?}", values);
        },
        local_ctrl_message::Payload::CmdSetPropVals(values) => {
            log::info!("values are {:?}", values);
        },
        _ => unreachable!(),
    }

    let msg_type = req_proto.msg();

    let res = match msg_type {
        LocalCtrlMsgType::TypeCmdGetPropertyCount => handle_cmd_get_property_count(),
        LocalCtrlMsgType::TypeCmdGetPropertyValues => handle_cmd_get_property_values(req_proto.payload.unwrap()),
        LocalCtrlMsgType::TypeCmdSetPropertyValues => handle_cmd_set_property_values(req_proto.payload.unwrap(), node.to_owned()),
        _ => vec![]
    };

    res
}

fn handle_cmd_get_property_count() -> Vec<u8> {
    let mut resp_payload = RespGetPropertyCount::default();
    resp_payload.status = Status::Success.into();
    resp_payload.count = 2;

    let mut resp = LocalCtrlMessage::default();
    resp.payload = Some(local_ctrl_message::Payload::RespGetPropCount(resp_payload));
    resp.encode_to_vec()
}

fn handle_cmd_get_property_values(req_payload: local_ctrl_message::Payload) -> Vec<u8> {
    let mut resp_payload = RespGetPropertyValues::default();

    match req_payload {
        local_ctrl_message::Payload::CmdGetPropVals(values) => {
            resp_payload.status = Status::Success.into();

            log::info!("{:?}", values.indices);
            for i in values.indices {
                let mut prop_info = PropertyInfo::default();
                prop_info.name = "Power".to_string();
                prop_info.r#type = 2;
                prop_info.flags = 0;
                prop_info.value = vec![0];
                log::info!("Get Property {} : {:?}", i, prop_info);
                resp_payload.props.push(prop_info);
            }

            let mut resp = LocalCtrlMessage::default();
            resp.payload = Some(local_ctrl_message::Payload::RespGetPropVals(resp_payload));
            resp.encode_to_vec()
        },
        _ => unreachable!()
    }
    
}

fn handle_cmd_set_property_values(req_payload: local_ctrl_message::Payload, node: Arc<Node<'_>>) -> Vec<u8> {
    let mut resp_payload = RespSetPropertyValues::default();

    match req_payload {
        local_ctrl_message::Payload::CmdSetPropVals(values) => {
            resp_payload.status = Status::Success.into();

            // log::info!("{:?}", values);
            log::info!("{:?}", std::str::from_utf8(&values.props[0].value).unwrap());

            let msg = values.props[0].value.clone();

            let received_val: HashMap<String, HashMap<String, Value>> =
                serde_json::from_str(&String::from_utf8(msg).unwrap()).unwrap();
            let devices = received_val.keys();
            for device in devices {
                let params = received_val.get(device).unwrap().to_owned();
                node.exeute_device_callback(&device, params);
            }

            let mut resp = LocalCtrlMessage::default();
            resp.payload = Some(local_ctrl_message::Payload::RespSetPropVals(resp_payload));
            resp.encode_to_vec()
        }
        _ => unreachable!() 
    }
}