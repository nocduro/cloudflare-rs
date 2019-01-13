use {Cloudflare, Error};

pub mod dns;
pub mod purge;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Plan {
    id: String,
    name: String,
    price: f64,
    currency: String,  // enum?
    frequency: String, // enum?
    legacy_id: String, // enum?
    is_subscribed: bool,
    can_subscribe: bool,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
    pub development_mode: u32,
    pub original_name_servers: Option<Vec<String>>,
    pub original_registrar: Option<String>,
    pub original_dnshost: Option<String>,
    pub created_on: String,
    pub modified_on: String,
    pub owner: HashMap<String, String>, // this should be its own `User` struct at some point
    pub permissions: Vec<String>,       // this should be a permission enum
    pub plan: Plan,
    pub plan_pending: Option<Plan>,
    pub status: Option<String>, // enum ?
    pub paused: Option<bool>,
    #[serde(rename = "type")]
    pub typex: Option<String>,
    pub name_servers: Vec<String>,
}

pub fn get_zoneid(api: &Cloudflare, domain: &str) -> Result<String, Error> {
    let zone: Vec<Zone> = api.make_get_req_param("zones", &[("name", domain)])?;
    if zone.len() < 1 {
        return Err(Error::InvalidOptions);
    }
    Ok(zone[0].id.clone())
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;
    use testenv::{API, DOMAIN};

    #[test]
    fn get_zone_id_test() {
        let _ = env_logger::try_init();
        let zone_id = get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();
        println!("zone id of {}: {}", DOMAIN.to_string(), zone_id);
    }
}
