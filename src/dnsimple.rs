use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ureq::{Request, Response};
use ureq::OrAnyStatus;
use crate::dnsimple::accounts::Accounts;
use crate::dnsimple::domains::Domains;
use crate::dnsimple::identity::Identity;
use crate::dnsimple::oauth::Oauth;

pub mod identity;
pub mod accounts;
pub mod oauth;
pub mod domains;

const VERSION: &str = "0.1.0";
const DEFAULT_USER_AGENT: &str = "dnsimple-rust/";

const API_VERSION: &str = "v2";
const DEFAULT_BASE_URL: &str = "https://api.dnsimple.com";
const DEFAULT_SANDBOX_URL: &str  = "https://api.sandbox.dnsimple.com";

/// Represents the Rust client for the DNSimple API V2
///
/// The client is your entrypoint to the DNSimple API. Using it
/// you will be able to call all the endpoints of the DNSimple API
/// and their respective functions.
///
/// # Examples
///
/// ```no_run
/// use dnsimple_rust::dnsimple::{Client, new_client};
///
/// let client = new_client(true, String::from("AUTH_TOKEN"));
/// let identity_response = client.identity().whoami().data;
///
/// match identity_response {
///         None => panic!("We should have a payload here."),
///         Some(whoami) =>  match whoami.data.account {
///             None => panic!("We should have the account data here"),
///             Some(account) => {
///             // so something with the account, like retrieving the id
///             // with account.id
///             }
///         }
/// }
pub struct Client {
    base_url: String,
    user_agent: String,
    auth_token: String,
    agent: ureq::Agent,
}

/// Represents the Error message payload returned by the DNSimple API
#[derive(Debug, Deserialize, Serialize)]
pub struct APIErrorMessage {
    pub error: String,
    pub error_description: String,
}

/// Represents a response from the DNSimple API
pub struct DNSimpleResponse<T> {
    pub rate_limit: String,
    pub rate_limit_remaining: String,
    pub rate_limit_reset: String,
    pub status: u16,
    pub data: Option<T>,
    pub message: Option<APIErrorMessage>
}

/// Represents an empty response from the DNSimple API
/// (_these type of responses happen when issuing DELETE commands for example_)
pub struct DNSimpleEmptyResponse {
    pub rate_limit: String,
    pub rate_limit_remaining: String,
    pub rate_limit_reset: String,
    pub status: u16,
}

/// Wrapper around a DNSimpleResponse and the raw http response of the DNSimple API
pub struct APIResponse<T> {
    pub response: DNSimpleResponse <T>,
    pub raw_http_response: Response
}

/// Helper function to create a new client
///
/// Make sure you use this to create your client.
///
/// # Examples
///
/// ```no_run
/// use dnsimple_rust::dnsimple::{Client, new_client};
///
/// let client = new_client(true, String::from("AUTH_TOKEN"));
/// ```
///
/// # Arguments
///
/// `sandbox`: `true` if you want to run in the sandbox environment, otherwise `false`
/// `token`: the bearer authentication token
pub fn new_client(sandbox: bool, token: String) -> Client {
    let mut url = DEFAULT_BASE_URL;
    if sandbox {
        url = DEFAULT_SANDBOX_URL;
    }
    Client {
        base_url: String::from(url),
        user_agent: DEFAULT_USER_AGENT.to_owned() + VERSION,
        auth_token: token,
        agent: ureq::Agent::new(),
    }
}

/// Helper function that will extract the `APIErrorMessage` from the raw http response.
///
/// # Arguments
///
/// `raw_response`: the raw http response to be parsed
pub fn dnsimple_error_from(raw_response: Response) -> Option<APIErrorMessage> {
    let raw_content = raw_response.into_string().unwrap();
    let tokens = raw_content.split("\n\n");
    let vec = tokens.collect::<Vec<&str>>();
    let body = vec.last().unwrap();
    let error_message: APIErrorMessage = serde_json::from_str(body).unwrap();
    Option::from(error_message)
}

impl Client {
    /// Returns the `accounts` service attached to this client
    pub fn accounts(&self) -> Accounts {
        Accounts {
            client: self
        }
    }

    /// Returns the `identity` service attached to this client
    pub fn identity(&self) -> Identity {
        Identity {
            client: self
        }
    }

    /// Returns the `oauth` service attached to this client
    pub fn oauth(&self) -> Oauth {
        Oauth {
            client: self
        }
    }

    /// Returns the `domains` service attached to this client
    pub fn domains(&self) -> Domains {
        Domains {
            client: self
        }
    }

    /// Convenience function to change the base url in runtime (used internally for
    /// testing).
    ///
    /// Note that if you want to do this you will have to declare your client mutable.
    ///
    /// ```no_run
    /// use dnsimple_rust::dnsimple::{Client, new_client};
    /// let mut client = new_client(true, String::from("ACCESS_TOKEN"));
    /// client.set_base_url("https://example.com");
    /// ```
    ///
    /// # Arguments
    ///
    /// `url`: The url we want to change the base url to.
    pub fn set_base_url(&mut self, url: &str) {
        self.base_url = String::from(url);
    }

    /// Returns the current url (including the `API_VERSION` as part of the path).
    pub fn versioned_url(&self) -> String {
        let mut url = String::from(&self.base_url);
        url.push_str("/");
        url.push_str(API_VERSION);
        url
    }

    /// Sends a GET request to the DNSimple API
    ///
    /// # Arguments
    ///
    /// `path`: the path to the endpoint
    pub fn get<T>(&self, path: &str) -> APIResponse<T> {
        let request = self.build_get_request(&path);

        let response = request.call();
        let dnsimple_response = Self::build_dnsimple_response(response.as_ref().unwrap());

        APIResponse {
            response: dnsimple_response,
            raw_http_response: response.unwrap()
        }
    }

    /// Sends a POST request to the DNSimple API
    ///
    /// # Arguments
    ///
    /// `path`: the path to the endpoint
    /// `data`: the json payload to be sent to the server
    pub fn post<T>(&self, path: &str, data: Value) -> APIResponse<T> {
        let request = self.build_post_request(&path);

        let response = request.send_json(data).or_any_status();
        let dnsimple_response = Self::build_dnsimple_response(&response.as_ref().unwrap());

        APIResponse {
            response: dnsimple_response,
            raw_http_response: response.unwrap()
        }
    }

    /// Sends a DELETE request to the DNSimple API
    ///
    /// # Arguments
    ///
    /// `path`: the path to the endpoint
    pub fn delete(&self, path: &str) -> DNSimpleEmptyResponse {
        let request = self.build_delete_request(&path);
        let response = request.call();

        Self::build_empty_dnsimple_response(response.as_ref().unwrap())
    }

    fn build_dnsimple_response<T>(response: &Response) -> DNSimpleResponse<T> {
        DNSimpleResponse {
            rate_limit: String::from(response.header("X-RateLimit-Limit").unwrap()),
            rate_limit_remaining: String::from(response.header("X-RateLimit-Remaining").unwrap()),
            rate_limit_reset: String::from(response.header("X-RateLimit-Reset").unwrap()),
            status: response.status(),
            data: None,
            message: None
        }
    }

    fn build_empty_dnsimple_response(response: &Response) -> DNSimpleEmptyResponse {
        DNSimpleEmptyResponse {
            rate_limit: String::from(response.header("X-RateLimit-Limit").unwrap()),
            rate_limit_remaining: String::from(response.header("X-RateLimit-Remaining").unwrap()),
            rate_limit_reset: String::from(response.header("X-RateLimit-Reset").unwrap()),
            status: response.status(),
        }
    }

    fn build_get_request(&self, path: &&str) -> Request {
        let request = self.agent.get(&self.url(path))
            .set("User-Agent", &self.user_agent)
            .set("Accept", "application/json");
        self.add_headers_to_request(request)
    }

    fn build_post_request(&self, path: &&str) -> Request {
        self.agent.post(&self.url(path))
            .set("User-Agent", &self.user_agent)
            .set("Accept", "application/json")
    }

    fn build_delete_request(&self, path: &&str) -> Request {
        let request = self.agent.delete(&self.url(path))
            .set("User-Agent", &self.user_agent)
            .set("Accept", "application/json");
        self.add_headers_to_request(request)
    }

    fn add_headers_to_request(&self, request: Request) -> Request {
        let auth_token = &format!("Bearer {}", self.auth_token);
        request
            .set("Authorization", auth_token.as_str())
    }

    fn url(&self, path: &str) -> String {
        let mut url = self.versioned_url();
        url.push_str(path);
        url
    }
}

#[cfg(test)]
mod tests {
    use crate::dnsimple::{DEFAULT_SANDBOX_URL, DEFAULT_USER_AGENT, new_client, VERSION};

    #[test]
    fn creates_a_client() {
        let token = "some-auth-token";
        let client = new_client(true, String::from(token));

        assert_eq!(client.base_url, DEFAULT_SANDBOX_URL);
        assert_eq!(client.user_agent, DEFAULT_USER_AGENT.to_owned() + VERSION);
        assert_eq!(client.auth_token, token);
    }

    #[test]
    fn can_change_the_base_url() {
        let mut client = new_client(true, String::from("token"));
        client.set_base_url("https://example.com");

        assert_eq!(client.versioned_url(), "https://example.com/v2");
    }
}