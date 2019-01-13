This library will no longer be updated since there is now an official cloudflare rust library: https://github.com/cloudflare/cloudflare-rs

### `cloudflare-rs` - Cloudflare api client written in Rust
Note: very little is actually done. 
I've only been adding stuff that is useful for myself, feel free to make a pull request with more endpoints added.

I've been making this library for my use in creating https://rustref.com [unfinished] so that new subdomain dns records can be added by a webserver when a configuration file is updated. Have to add new CNAME for each subdomain redirect because Cloudflare only allows proxied wildcard CNAME records for enterprise customers.

The design of this crate is inspired by the `gitlab` crate: https://crates.io/crates/gitlab

License: MIT
