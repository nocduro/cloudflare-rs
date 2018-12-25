use {Cloudflare, Error};

#[derive(Deserialize, Debug)]
pub struct User {
    id: String,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    username: String,
    telephone: Option<String>,
    country: Option<String>,
    zipcode: Option<String>,
    created_on: String,
    modified_on: String,
    two_factor_authentication_enabled: bool,
    two_factor_authentication_locked: bool,
    organizations: Vec<String>,
    has_pro_zones: bool,
    has_business_zones: bool,
    has_enterprise_zones: bool,
}

pub fn user_details(api: &Cloudflare) -> Result<User, Error> {
    Ok(api.make_get_req("user")?)
}

#[allow(unused)]
pub fn update_user(api: &Cloudflare, user: &User) -> Result<(), Error> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use testenv::API;

    #[test]
    fn test_user_details() {
        assert!(user_details(&API).is_ok());
    }
}
