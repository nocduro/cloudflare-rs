#[macro_use]
extern crate hyper;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

#[cfg(test)]
extern crate dotenv;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

use std::io::Read;

use serde::de::DeserializeOwned;
use std::fmt::Debug;

use self::url::Url;

pub mod user_api;

header! { (XAuthKey, "X-Auth-Key") => [String] }
header! { (XAuthEmail, "X-Auth-Email") => [String] }

#[derive(Debug)]
pub struct Cloudflare {
    api_key: String,
    api_email: String,
    api_user_service_key: Option<String>,
    organization_id: Option<String>,
    client: reqwest::Client,
    base_url: Url,
    auth_type: AuthType,
}

#[derive(Deserialize, Debug)]
struct ResultInfo {
    page: u32,
    per_page: u32,
    count: u32,
    total_count: u32,
}

#[derive(Deserialize, Debug)]
struct ErrorDescription {
    code: u32,
    message: String,
}

#[derive(Deserialize, Debug)]
struct Response<T> {
    result: Option<T>,
    success: bool,
    errors: Vec<ErrorDescription>,
    messages: Vec<String>,
    result_info: Option<ResultInfo>,
}

#[derive(Debug)]
pub enum AuthType {
    AuthKeyEmail,
    AuthUserService,
}

#[derive(Debug)]
pub enum Error {
    InvalidOptions,
    NotSuccess,
}

impl Cloudflare {
    pub fn new(key: &str, email: &str, base_url: &str) -> Result<Cloudflare, Error> {
        Ok(Cloudflare {
            api_key: key.to_string(),
            api_email: email.to_string(),
            api_user_service_key: None,
            organization_id: None,
            client: reqwest::Client::new(),
            base_url: Url::parse(base_url).map_err(|_| Error::InvalidOptions)?,
            auth_type: AuthType::AuthKeyEmail,
        })
    }

    fn execute_request<T>(&self, method: reqwest::Method, url: Url) -> Result<Response<T>, Error>
    where
        T: DeserializeOwned,
    {
        let mut request = reqwest::Request::new(method, url);
        request.headers_mut().set(XAuthKey(self.api_key.clone()));
        request
            .headers_mut()
            .set(XAuthEmail(self.api_email.clone()));

        let mut response = self.client.execute(request).unwrap();

        // read in response, and deserialize
        let mut response_json = String::new();
        response
            .read_to_string(&mut response_json)
            .expect("read to string error");

        let parsed: Response<T> = serde_json::from_str(&response_json).expect("parse error");
        if !parsed.success {
            return Err(Error::NotSuccess);
        }

        Ok(parsed)
    }

    fn make_request<T>(&self, method: reqwest::Method, path: &str) -> Result<T, Error>
    where
        T: Debug + DeserializeOwned,
    {
        // construct the url we want to contact
        let url_path = self.base_url.join(path).unwrap();
        Ok(self.execute_request(method, url_path)
            .unwrap()
            .result
            .unwrap())
    }

    #[allow(unused)]
    fn make_request_params<T>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T, Error>
    where
        T: Debug + DeserializeOwned,
    {
        // construct the url we want to contact
        let mut url_path = self.base_url.join(path).unwrap();
        url_path.query_pairs_mut().clear();
        params.iter().for_each(|&(k, v)| {
            url_path.query_pairs_mut().append_pair(k, v);
        });

        Ok(self.execute_request(method, url_path)
            .unwrap()
            .result
            .unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dotenv;

    lazy_static! {
        static ref API_KEY: String = dotenv::var("cloudflare_key").unwrap();
        static ref EMAIL: String = dotenv::var("email").unwrap();
        pub static ref API: Cloudflare = Cloudflare::new(&API_KEY, &EMAIL, "https://api.cloudflare.com/client/v4/").unwrap();
    }

    #[test]
    fn api_key_loaded() {
        assert!(API_KEY.len() > 0);
    }
}
