use std::sync::Arc;

use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize};

use crate::providers::vultr::models::request::instance::InstanceBuilder;
use crate::providers::vultr::models::request::region::Region;
use crate::providers::vultr::models::request::plan::Plan;
use crate::shared_config::SharedConfig;
use crate::utils::error::ServiceError;

#[derive(Debug, Deserialize)]
struct InstanceParams {
    region: Region,
    plan: Plan,
    os_id: u32,
    ipxe_chain_url: Option<String>,
    iso_id: Option<String>,
    script_id: Option<String>,
    snapshot_id: Option<String>,
    enable_ipv6: Option<bool>,
    attach_private_network_deprecated: Option<Vec<String>>,
    attach_vpc: Option<Vec<String>>,
    label: Option<String>,
    sshkey_id: Option<Vec<String>>,
    backups: Option<String>,
    app_id: Option<u32>,
    image_id: Option<String>,
    user_data: Option<String>,
    ddos_protection: Option<bool>,
    activation_email: Option<bool>,
    hostname: Option<String>,
    tag: Option<String>,
    firewall_group_id: Option<String>,
    reserved_ipv4: Option<String>,
    enable_private_network_deprecated: Option<bool>,
    enable_vpc: Option<bool>,
    tags: Option<Vec<String>>,
}

pub fn create_instance(data: web::Json<InstanceParams>, config: web::Data<Arc<SharedConfig>>) -> impl Responder {
    let params = data.into_inner();
    let shared_config = config.get_ref().clone();

    let instance = InstanceBuilder::new()
        .region(params.region)
        .plan(params.plan)
        .os_id(params.os_id)
        .ipxe_chain_url(params.ipxe_chain_url.unwrap_or_default())
        .iso_id(params.iso_id.unwrap_or_default())
        .script_id(params.script_id.unwrap_or_default())
        .build(shared_config.clone());

     match instance {
        Ok(instance) => Ok(HttpResponse::Ok().json(instance)),
        Err(e) => Err(ServiceError::from(e)),
    }
}