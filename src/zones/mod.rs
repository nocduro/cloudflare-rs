use {Cloudflare, Error};
use reqwest::Method::Get;

pub mod dns;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Plan {
    id: String,
    name: String,
    price: f64,
    currency: String, // enum?
    frequency: String, // enum?
    legacy_id: String, // enum?
    is_subscribed: bool,
    can_subscribe: bool,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    id: String,
    development_mode: u32,
    original_name_servers: Option<Vec<String>>,
    original_registrar: Option<String>,
    original_dnshost: Option<String>,
    created_on: String,
    modified_on: String,
    owner: HashMap<String, String>, // this should be its own `User` struct at some point
    permissions: Vec<String>, // this should be a permission enum
    plan: Plan,
    plan_pending: Option<Plan>,
    status: Option<String>, // enum ?
    paused: Option<bool>,
    #[serde(rename = "type")]
    typex: Option<String>,
    name_servers: Vec<String>,
}

pub fn get_zoneid(api: &Cloudflare, domain: &str) -> Result<String, Error> {
    let zone: Vec<Zone> = api.make_request_params(Get, "zones", &[("name", domain)])?;
    if zone.len() < 1 {
        return Err(Error::InvalidOptions)
    }
    Ok(zone[0].id.clone())    
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use testenv::{API, DOMAIN};
    use super::*;

    #[test]
    fn get_zone_id_test() {
        let _ = env_logger::try_init();
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();
        println!("zone id of {}: {}", DOMAIN.to_string(), zone_id);
    }
}
