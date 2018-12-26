use {Cloudflare, Error, Response};

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
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

/// The direction that a result should be sorted in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    /// A-Z (Default)
    Ascending,

    /// Z-A
    Descending,
}

/// The field that a record list should be sorted on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListQuerySorting {
    Type(SortDirection),
    Name(SortDirection),
    Content(SortDirection),
    TTL(SortDirection),
    Proxied(SortDirection),
}

/// If `all` conditions or just `any` condition must match for a DNS record to be found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
    /// All matchers must be fulfilled to match a record. (Default)
    All,

    /// Any of the specified matchers must match for a record to be returned.
    Any,
}

/// Query parameters for listing DNS records. Values left as None will be left out and the default
/// value from the server will be used.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ListQueryParams<'a> {
    record_type: Option<RecordType>,
    name: Option<&'a str>,
    content: Option<&'a str>,
    page: Option<&'a str>, // TODO: Refactor into real numbers
    per_page: Option<&'a str>,
    sort: Option<ListQuerySorting>,
    match_type: MatchType,
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
        self.as_ref().fmt(f)
    }
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::All
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

impl ListQuerySorting {
    pub fn by_type() -> Self {
        ListQuerySorting::Type(SortDirection::default())
    }

    pub fn by_name() -> Self {
        ListQuerySorting::Name(SortDirection::default())
    }

    pub fn by_content() -> Self {
        ListQuerySorting::Content(SortDirection::default())
    }

    pub fn by_ttl() -> Self {
        ListQuerySorting::TTL(SortDirection::default())
    }

    pub fn by_proxied() -> Self {
        ListQuerySorting::Proxied(SortDirection::default())
    }

    pub fn asc(self) -> Self {
        match self {
            ListQuerySorting::Type(_) => ListQuerySorting::Type(SortDirection::Ascending),
            ListQuerySorting::Name(_) => ListQuerySorting::Name(SortDirection::Ascending),
            ListQuerySorting::Content(_) => ListQuerySorting::Content(SortDirection::Ascending),
            ListQuerySorting::TTL(_) => ListQuerySorting::TTL(SortDirection::Ascending),
            ListQuerySorting::Proxied(_) => ListQuerySorting::Proxied(SortDirection::Ascending),
        }
    }

    pub fn desc(self) -> Self {
        match self {
            ListQuerySorting::Type(_) => ListQuerySorting::Type(SortDirection::Descending),
            ListQuerySorting::Name(_) => ListQuerySorting::Name(SortDirection::Descending),
            ListQuerySorting::Content(_) => ListQuerySorting::Content(SortDirection::Descending),
            ListQuerySorting::TTL(_) => ListQuerySorting::TTL(SortDirection::Descending),
            ListQuerySorting::Proxied(_) => ListQuerySorting::Proxied(SortDirection::Descending),
        }
    }

    pub fn direction_str(&self) -> &str {
        match self {
            ListQuerySorting::Type(direction) => direction.as_ref(),
            ListQuerySorting::Name(direction) => direction.as_ref(),
            ListQuerySorting::Content(direction) => direction.as_ref(),
            ListQuerySorting::TTL(direction) => direction.as_ref(),
            ListQuerySorting::Proxied(direction) => direction.as_ref(),
        }
    }
}

impl AsRef<str> for RecordType {
    fn as_ref(&self) -> &'static str {
        match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
            RecordType::CNAME => "CNAME",
            RecordType::TXT => "TXT",
            RecordType::SRV => "SRV",
            RecordType::LOC => "LOC",
            RecordType::MX => "MX",
            RecordType::NS => "NS",
            RecordType::SPF => "SPF",
        }
    }
}

impl AsRef<str> for ListQuerySorting {
    fn as_ref(&self) -> &'static str {
        match self {
            ListQuerySorting::Type(_) => "type",
            ListQuerySorting::Name(_) => "name",
            ListQuerySorting::Content(_) => "content",
            ListQuerySorting::TTL(_) => "ttl",
            ListQuerySorting::Proxied(_) => "proxied",
        }
    }
}

impl AsRef<str> for SortDirection {
    fn as_ref(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "asc",
            SortDirection::Descending => "desc",
        }
    }
}

impl AsRef<str> for MatchType {
    fn as_ref(&self) -> &'static str {
        match self {
            MatchType::All => "all",
            MatchType::Any => "any",
        }
    }
}

impl<'a> ListQueryParams<'a> {
    pub fn as_query_params(&'a self) -> Vec<(&'a str, &'a str)> {
        let mut params = vec![];

        if let Some(ref record_type) = self.record_type {
            params.push(("type", record_type.as_ref()));
        }

        if let Some(ref name) = self.name {
            params.push(("name", name));
        }

        if let Some(ref content) = self.content {
            params.push(("content", content));
        }

        if let Some(ref page) = self.page {
            params.push(("page", page));
        }

        if let Some(ref per_page) = self.per_page {
            params.push(("per_page", per_page));
        }

        if let Some(ref sort) = self.sort {
            params.push(("order", sort.as_ref()));
            params.push(("direction", sort.direction_str()));
        }

        params.push(("match", self.match_type.as_ref()));

        params
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

pub fn list_records<'a, Q: Into<&'a ListQueryParams<'a>>>(
    api: &Cloudflare,
    zone: &str,
    query: Q,
) -> Result<Vec<DnsRecord>, Error> {
    let response: Response<Vec<DnsRecord>> = api.get_params(
        &format!("zones/{}/dns_records", zone),
        &query.into().as_query_params(),
    )?;

    if !response.success {
        return Err(Error::NotSuccess);
    }

    Ok(response.result.unwrap_or_default())
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
    fn get_all_dns() {
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
    fn get_dns() {
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN);
        assert!(zone_id.is_ok());
        let zone_id = zone_id.unwrap();

        let records = list_records(
            &API,
            &zone_id,
            &ListQueryParams {
                sort: Some(ListQuerySorting::by_type()),
                ..Default::default()
            },
        );
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

    #[test]
    fn it_has_correct_defaults() {
        assert_eq!(MatchType::default(), MatchType::All);
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn it_has_fluent_sorting_options() {
        assert_eq!(
            ListQuerySorting::by_type().desc(),
            ListQuerySorting::Type(SortDirection::Descending)
        );

        assert_eq!(
            ListQuerySorting::by_ttl().asc(),
            ListQuerySorting::TTL(SortDirection::Ascending)
        );
        assert_eq!(
            ListQuerySorting::by_content(),
            ListQuerySorting::Content(SortDirection::Ascending)
        );
    }

    #[test]
    fn it_emits_correct_strings_for_sorting_options() {
        assert_eq!(
            ListQuerySorting::Type(SortDirection::default()).as_ref(),
            "type",
        );
        assert_eq!(
            ListQuerySorting::Name(SortDirection::default()).as_ref(),
            "name",
        );
        assert_eq!(
            ListQuerySorting::Content(SortDirection::default()).as_ref(),
            "content",
        );
        assert_eq!(
            ListQuerySorting::TTL(SortDirection::default()).as_ref(),
            "ttl",
        );
        assert_eq!(
            ListQuerySorting::Proxied(SortDirection::default()).as_ref(),
            "proxied",
        );

        assert_eq!(
            ListQuerySorting::Name(SortDirection::Ascending).direction_str(),
            "asc",
        );
        assert_eq!(
            ListQuerySorting::Name(SortDirection::Descending).direction_str(),
            "desc",
        );
    }

    #[test]
    fn it_generates_query_parameters() {
        let query = ListQueryParams {
            record_type: Some(RecordType::CNAME),
            name: None,
            content: Some("example.com"),
            page: Some("2"),
            per_page: Some("5"),
            sort: Some(ListQuerySorting::by_name().desc()),
            match_type: MatchType::Any,
        };
        let query_params = query.as_query_params();
        assert_eq!(
            query_params,
            vec![
                ("type", "CNAME"),
                ("content", "example.com"),
                ("page", "2"),
                ("per_page", "5"),
                ("order", "name"),
                ("direction", "desc"),
                ("match", "any"),
            ],
        );
    }

    #[test]
    fn it_has_sane_defaults_for_list_query() {
        assert_eq!(
            ListQueryParams::default(),
            ListQueryParams {
                record_type: None,
                name: None,
                content: None,
                page: None,
                per_page: None,
                sort: None,
                match_type: MatchType::All,
            }
        );
    }
}
