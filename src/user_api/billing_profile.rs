use {Cloudflare, Error};
use reqwest::Method::Get;

#[derive(Deserialize, Debug)]
pub struct BillingProfile {
    id: String,
    first_name: String,
    last_name: String,
    address: String,
    address2: String,
    company: String,
    city: String,
    state: String,
    zipcode: String,
    country: String,
    telephone: String,
    card_number: String,
    card_expiry_year: u32,
    card_expiry_month: u8,
    vat: String,
    edited_on: String,
    created_on: String,
}

pub fn billing_profile(api: &Cloudflare) -> Result<BillingProfile, Error> {
    Ok(api.make_request(Get, "user/billing/profile")?)
}

#[cfg(test)]
mod tests {
    use testenv::API;
    use super::*;

    #[test]
    fn test_billing_profile() {
        assert!(billing_profile(&API).is_ok());
    }
}