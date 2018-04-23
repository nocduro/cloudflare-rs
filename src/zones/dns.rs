use {Cloudflare, Error};

use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    TXT,
    SRV,
    LOC,
    MX,
    NS,
    SPF,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: RecordType,
    pub name: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: u32,
    pub locked: bool,
    pub zone_id: String,
    pub zone_name: String,
    pub created_on: String,
    pub modified_on: String,
    pub data: Option<HashMap<String, String>>,
}

/// Need `Display` so we can call `.to_string()` when dealing with the API.
impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub fn create_dns_entry(
    api: &Cloudflare,
    zone: &str,
    dns_type: RecordType,
    name: &str,
    content: &str,
) -> Result<DnsRecord, Error> {
    let json = json!({
        "name": name,
        "content": content,
        "type": dns_type.to_string(),
    });
    Ok(api.make_post_req(&format!("zones/{}/dns_records", zone), json)?)
}

// TODO: refactor this into builder pattern, so it is easy to customize the record
// TODO: use structs instead of hardcoded json!() ?
pub fn create_proxied_dns_entry(
    api: &Cloudflare,
    zone: &str,
    dns_type: RecordType,
    name: &str,
    content: &str,
) -> Result<DnsRecord, Error> {
    let json = json!({
        "name": name,
        "content": content,
        "type": dns_type.to_string(),
        "proxied": true,
        "ttl": 1,
    });
    Ok(api.make_post_req(&format!("zones/{}/dns_records", zone), json)?)
}

pub fn list_dns_records(api: &Cloudflare, zone: &str) -> Result<Vec<DnsRecord>, Error> {
    Ok(api.get_all(&format!("zones/{}/dns_records", zone))?)
}

pub fn list_dns_of_type(
    api: &Cloudflare,
    zone: &str,
    record_type: RecordType,
) -> Result<Vec<DnsRecord>, Error> {
    Ok(api.get_all_params(
        &format!("zones/{}/dns_records", zone),
        &[("type", &record_type.to_string())],
    )?)
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use testenv::{API, DOMAIN};
    use super::*;

    #[test]
    fn get_dns() {
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();

        let records = list_dns_records(&API, &zone_id);
        assert!(records.is_ok());
        let records = records.unwrap();
        // println!("{} records: {:#?}", records.len(), records);
        assert!(records.len() > 0);
    }

    #[test]
    fn get_cname() {
        let _ = env_logger::try_init();
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();

        let records = list_dns_of_type(&API, &zone_id, RecordType::CNAME);
        assert!(records.is_ok());
        let records = records.unwrap();
        // println!("{} records: {:#?}", records.len(), records);
        assert!(records.len() > 0);
    }

    #[test]
    #[ignore]
    fn create_cname() {
        let _ = env_logger::try_init();
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();

        println!("{}", RecordType::CNAME.to_string());
        let created = create_proxied_dns_entry(
            &API,
            &zone_id,
            RecordType::CNAME,
            "unit.nocduro.com",
            "nocduro.com",
        );
        println!("{:#?}", created);
        assert!(created.is_ok());
    }
}
