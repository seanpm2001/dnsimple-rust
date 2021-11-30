use crate::dnsimple::{Client, DNSimpleEmptyResponse, DNSimpleResponse, Endpoint};
use serde_json::Value;
use crate::dnsimple::registrar_name_servers::VanityNameServer;

struct VanityNameServersEndpoint;

impl Endpoint for VanityNameServersEndpoint {
    type Output = Vec<VanityNameServer>;
}

pub struct VanityNameServers<'a> {
    pub client: &'a Client
}

impl VanityNameServers<'_> {
    /// Enable vanity name servers
    ///
    /// # Arguments
    /// `account_id`: The account id
    /// `domain`: The domain name or id
    pub fn enable_vanity_name_servers(&self, account_id: u64, domain: String) -> Result<DNSimpleResponse<Vec<VanityNameServer>>, String> {
        let path = format!("/{}/vanity/{}", account_id, domain);

        self.client.put::<VanityNameServersEndpoint>(&path, Value::Null)
    }

    /// Enable vanity name servers
    ///
    /// # Arguments
    /// `account_id`: The account id
    /// `domain`: The domain name or id
    pub fn disable_vanity_name_servers(&self, account_id: u64, domain: String) -> DNSimpleEmptyResponse {
        let path = format!("/{}/vanity/{}", account_id, domain);

        self.client.delete(&path)
    }
}