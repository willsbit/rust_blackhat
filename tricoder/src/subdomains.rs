use std::collections::HashSet;
use std::time::Duration;
use crate::error::Error;
use crate::model::{Subdomain, CrtShEntry};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};
use reqwest::blocking::Client;


/// This tool will be using the API provided by crt.sh,
/// which can be queried in the endpoint
/// `https://crt.sh/?q=%25.[domain.com]&output=json"`.

pub fn enumerate(http_client: &Client,
                 target: &str)
    -> Result<Vec<Subdomain>, Error>
{
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh/?q=%25.{}&output=json", target))
        .send()?
        .json()?;

    // clean and deduplicate results
    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| {
            entry
                .name_value
                .split("\n")
                .map(|subdomain| subdomain.trim().to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains("*"))
        .collect();

    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter(resolves)
        .collect();

    Ok(subdomains)
}

pub fn resolves(domain: &Subdomain) -> bool {
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(4);

    let dns_resolver = Resolver::new(
        ResolverConfig::default(),
        opts,
    )
        .expect("subdomain resolver: building DNS client");
    dns_resolver.lookup_ip(domain.domain.as_str()).is_ok()
}