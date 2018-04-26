#[macro_use]
extern crate hyper;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate url;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate dotenv;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

use std::io::Read;

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;

use self::url::Url;

pub mod user_api;
pub mod zones;
pub mod errors;

pub use errors::{Error, Result};

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

impl Cloudflare {
    pub fn new(key: &str, email: &str, base_url: &str) -> Result<Cloudflare> {
        Ok(Cloudflare {
            api_key: key.to_string(),
            api_email: email.to_string(),
            api_user_service_key: None,
            organization_id: None,
            client: reqwest::Client::new(),
            base_url: Url::parse(base_url)?,
            auth_type: AuthType::AuthKeyEmail,
        })
    }

    fn execute_request<T>(&self, mut request: reqwest::RequestBuilder) -> Result<Response<T>>
    where
        T: Debug + DeserializeOwned,
    {
        let mut response = request.header(XAuthKey(self.api_key.clone()))
            .header(XAuthEmail(self.api_email.clone()))
            .send()?;

        // read in response, and deserialize
        let mut response_json = String::new();
        response.read_to_string(&mut response_json)?;

        // println!("response_json: {}", response_json);
        let parsed: Response<T> = serde_json::from_str(&response_json)?;
        if !parsed.success {
            // handle Cloudflare specific errors here
            debug!("parsed struct: {:?}", parsed);
            return Err(Error::NotSuccess);
        }

        Ok(parsed)
    }

    fn execute_post_req<T>(&self, url: Url, body: Value) -> Result<Response<T>>
    where T: Debug + DeserializeOwned
    {
        let mut req = self.client.post(url);
        req.json(&body);
        self.execute_request(req)
    }
    
    fn make_post_req<T>(&self, path: &str, body: Value) -> Result<T>
    where T: Debug + DeserializeOwned
    {
        let url = self.base_url.join(path)?;
        self.execute_post_req(url, body)?.result.ok_or(Error::NotSuccess)
    }

    fn execute_get_req<T>(&self, url: Url) -> Result<Response<T>>
    where T: Debug + DeserializeOwned
    {
        let req = self.client.get(url);
        self.execute_request(req)
    }

    fn make_get_req<T>(&self, path: &str) -> Result<T>
    where T: Debug + DeserializeOwned
    {
        let url = self.base_url.join(path)?;
        self.execute_get_req(url)?.result.ok_or(Error::NotSuccess)
    }

    fn make_get_req_param<T>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> 
    where T: Debug + DeserializeOwned
    {
        // construct the url we want to contact
        let mut url = self.base_url.join(path)?;
        url.query_pairs_mut().clear();
        params.iter().for_each(|&(k, v)| {
            url.query_pairs_mut().append_pair(k, v);
        });
        self.execute_get_req(url)?.result.ok_or(Error::NotSuccess)
    }

    fn make_delete_req<T>(&self, path: &str, body: Value) -> Result<T> 
    where T: Debug + DeserializeOwned
    {
        // construct the url we want to contact
        let url = self.base_url.join(path)?;
        self.execute_delete_req(url, body)?.result.ok_or(Error::NotSuccess)
    }

    fn execute_delete_req<T>(&self, url: Url, body: Value) -> Result<Response<T>>
    where T: Debug + DeserializeOwned
    {
        let mut req = self.client.delete(url);
        req.json(&body);
        self.execute_request(req)
    }

    fn get_all<T>(&self, path: &str) -> Result<Vec<T>>
    where
        T: Debug + DeserializeOwned,
    {
        let mut page_num = 1u32;
        let mut url_path = self.base_url.join(path)?;
        let mut output: Vec<T> = Vec::new();

        loop {
            // build the url for that page
            // TODO: clean this up?

            url_path.set_query(Some(&format!("page={}", page_num)));
            let resp: Response<Vec<T>> =
                self.execute_get_req(url_path.clone())?;
            if !resp.success {
                return Err(Error::NotSuccess);
            }

            output.extend(resp.result.ok_or(Error::NoResultsReturned)?);
            page_num += 1;

            // check if we received all of the elements
            let page_info = &resp.result_info.ok_or(Error::NoResultsReturned)?;
            debug!("page_info: {:?}", page_info);
            if page_info.count < page_info.per_page
                || page_info.page * page_info.per_page == page_info.total_count
            {
                return Ok(output);
            }
        }
    }

    fn get_all_params<T>(&self, path: &str, params: &[(&str, &str)]) -> Result<Vec<T>>
    where
        T: Debug + DeserializeOwned,
    {
        if params.iter().any(|&(k, _)| k == "page") {
            return Err(Error::InvalidOptions);
        }
        let mut page_num = 1u32;

        // construct the url we want to contact with the passed in params
        let mut url_path = self.base_url.join(path)?;
        url_path.query_pairs_mut().clear();
        params.iter().for_each(|&(k, v)| {
            url_path.query_pairs_mut().append_pair(k, v);
        });
        let orig_query = url_path.query().ok_or(Error::NoResultsReturned)?.to_string();

        let mut output: Vec<T> = Vec::new();

        loop {
            // build the url for that page
            // TODO: clean this up?
            debug!("Getting page {}", page_num);
            url_path.set_query(Some(&format!("{}&page={}", orig_query, page_num)));
            let resp: Response<Vec<T>> =
                self.execute_get_req(url_path.clone())?;
            if !resp.success {
                return Err(Error::NotSuccess);
            }

            output.extend(resp.result.ok_or(Error::NoResultsReturned)?);
            page_num += 1;

            // check if we received all of the elements
            let page_info = &resp.result_info.ok_or(Error::NoResultsReturned)?;
            debug!("page_info: {:?}", page_info);
            if page_info.count < page_info.per_page
                || page_info.page * page_info.per_page == page_info.total_count
            {
                return Ok(output);
            }
        }
    }
}

#[cfg(test)]
mod testenv {
    use super::*;
    use dotenv;

    lazy_static! {
        static ref API_KEY: String = dotenv::var("cloudflare_key").unwrap();
        static ref EMAIL: String = dotenv::var("cloudflare_email").unwrap();
        pub static ref DOMAIN: String = dotenv::var("domain").unwrap();
        pub static ref API: Cloudflare =
            Cloudflare::new(&API_KEY, &EMAIL, "https://api.cloudflare.com/client/v4/").unwrap();
    }

    #[test]
    fn api_key_loaded() {
        assert!(API_KEY.len() > 0);
    }
}
