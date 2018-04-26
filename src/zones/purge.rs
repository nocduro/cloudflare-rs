use {Cloudflare, Error};

/// Remove all resources from Cloudflare's cache.
///
/// Note: This may have dramatic affects on your origin server load after 
/// performing this action.
pub fn purge_everything(api: &Cloudflare, zone_id: &str) -> Result<(), Error> {
    let json = json!({"purge_everything": true});
    Ok(api.make_delete_req(&format!("zones/{}/purge_cache", zone_id), json)?)
}
