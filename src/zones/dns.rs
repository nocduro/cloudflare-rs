use {Cloudflare, Error};

use std::collections::HashMap;
use std::fmt;

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

/// Payload for a DNS update. Optional fields can be left as `None` to not change them from the
/// currently live values.
#[derive(Debug, Serialize)]
pub struct UpdateDnsRecord {
    #[serde(rename = "type")]
    pub record_type: RecordType,
    pub name: String,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
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

pub fn update_dns_entry(
    api: &Cloudflare,
    zone: &str,
    id: &str,
    payload: &UpdateDnsRecord,
) -> Result<DnsRecord, Error> {
    Ok(api.make_put_req(
        &format!("zones/{}/dns_records/{}", zone, id),
        serde_json::to_value(payload)?,
    )?)
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
    use super::*;
    use testenv::{API, DOMAIN};

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

    #[test]
    fn update_cname() {
        let _ = env_logger::try_init();
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();

        let records = list_dns_of_type(&API, &zone_id, RecordType::CNAME);
        assert!(records.is_ok());
        let records = records.unwrap();
        let record = records
            .first()
            .expect("No CNAMEs set on the server. Please re-run tests.");

        let result = update_dns_entry(
            &API,
            &zone_id,
            &record.id,
            &UpdateDnsRecord {
                record_type: RecordType::CNAME,
                name: record.name.clone(),
                content: record.content.clone(),
                ttl: Some(record.ttl),
                proxied: None,
            },
        );
        assert!(result.is_ok());
    }
}
