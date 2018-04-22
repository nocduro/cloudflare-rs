use {Cloudflare, Error};
use reqwest::Method::Post;

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
    id: String,
    #[serde(rename = "type")]
    record_type: RecordType,
    name: String,
    content: String,
    proxiable: bool,
    proxied: bool,
    ttl: u32,
    locked: bool,
    zone_id: String,
    zone_name: String,
    created_on: String,
    modified_on: String,
    data: Option<HashMap<String, String>>,
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
    Ok(api.make_request_params(
        Post,
        &format!("zones/{}/dns_records", zone),
        &[
            ("type", &dns_type.to_string()),
            ("name", name),
            ("content", content),
        ],
    )?)
}

pub fn list_dns_records(api: &Cloudflare, zone: &str) -> Result<Vec<DnsRecord>, Error> {
    Ok(api.get_all(&format!("zones/{}/dns_records", zone))?)
}

pub fn list_dns_of_type(api: &Cloudflare, zone: &str, record_type: RecordType) -> Result<Vec<DnsRecord>, Error> {
    Ok(api.get_all_params(&format!("zones/{}/dns_records", zone), &[("type", &record_type.to_string())])?)
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
        println!("{} records: {:#?}", records.len(), records);
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
        println!("{} records: {:#?}", records.len(), records);
        assert!(records.len() > 0);
    }
}
