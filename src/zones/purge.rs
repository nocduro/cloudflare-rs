use serde_json::Value;
use {Cloudflare, Error};
/// Remove all resources from Cloudflare's cache.
///
/// Note: This may have dramatic affects on your origin server load after
/// performing this action.
pub fn purge_everything(api: &Cloudflare, zone_id: &str) -> Result<Value, Error> {
    let json = json!({"purge_everything": true});
    Ok(api.make_delete_req(&format!("zones/{}/purge_cache", zone_id), json)?)
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;
    use testenv::{API, DOMAIN};

    #[test]
    #[ignore]
    fn purge() {
        let zone_id = ::zones::get_zoneid(&API, &DOMAIN).unwrap();
        let purge = purge_everything(&API, &zone_id);
        println!("purge: {:?}", purge);
        assert!(purge.is_ok());
    }
}
