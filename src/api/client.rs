//! HTTP client targeting the Nubster control-plane API.

#![allow(dead_code)] // Surface wired into commands by #8-#9; drop this allow there.

use std::time::Duration;

use reqwest::{Response, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::auth::token_store::TokenStore;
use crate::config::Config;
use crate::error::CliError;

/// Total request timeout applied to every call.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Bounded exponential-backoff policy for transient failures.
pub struct RetryPolicy {
    /// Maximum number of attempts, including the first.
    pub max_attempts: u32,
    /// Delay before the first retry; doubled on each subsequent attempt.
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
        }
    }
}

/// Client for the control-plane REST API, authenticated with a bearer token.
pub struct Client {
    http: reqwest::Client,
    base_url: Url,
    token: String,
    retry: RetryPolicy,
}

impl Client {
    /// Builds a client from an explicit base URL and bearer token.
    ///
    /// # Errors
    /// Returns [`CliError`] when `base_url` is not a valid URL or the
    /// underlying HTTP client cannot be constructed.
    pub fn new(base_url: &str, token: &str) -> Result<Self, CliError> {
        let mut base_url = Url::parse(base_url)
            .map_err(|e| CliError::Generic(format!("invalid base URL `{base_url}`: {e}")))?;
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| CliError::Generic(format!("cannot build HTTP client: {e}")))?;
        Ok(Self {
            http,
            base_url,
            token: token.to_owned(),
            retry: RetryPolicy::default(),
        })
    }

    /// Builds a client by resolving the base URL from `config` for `host`
    /// and the bearer token from `store`.
    ///
    /// # Errors
    /// Returns [`CliError::NotAuthenticated`] when no token is stored for
    /// `host`, or [`CliError`] on a configuration or backend failure.
    pub fn for_host(config: &Config, host: &str, store: &TokenStore) -> Result<Self, CliError> {
        let base_url = config
            .hosts
            .get(host)
            .map_or_else(|| format!("https://{host}"), |h| h.api_url.clone());
        let token = store.get(host)?.ok_or(CliError::NotAuthenticated)?;
        Self::new(&base_url, &token)
    }

    /// Replaces the retry policy, chiefly for tests.
    #[must_use]
    pub fn with_retry(self, retry: RetryPolicy) -> Self {
        Self { retry, ..self }
    }

    /// Sends a GET request to `path` and deserializes a 2xx JSON body.
    ///
    /// # Errors
    /// Returns [`CliError`] on a transport failure or a non-success status.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, CliError> {
        let url = self.url_for(path)?;
        let builder = self.http.get(url).bearer_auth(&self.token);
        let response = self.send_with_retry(builder).await?;
        map_response(response).await
    }

    /// Sends a POST request to `path` with a JSON `body` and deserializes a
    /// 2xx JSON response.
    ///
    /// # Errors
    /// Returns [`CliError`] on a transport failure or a non-success status.
    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CliError> {
        let url = self.url_for(path)?;
        let builder = self.http.post(url).bearer_auth(&self.token).json(body);
        let response = self.send_with_retry(builder).await?;
        map_response(response).await
    }

    /// Joins `path` onto the base URL, tolerating a leading slash.
    fn url_for(&self, path: &str) -> Result<Url, CliError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|e| CliError::Generic(format!("invalid request path `{path}`: {e}")))
    }

    /// Sends `builder`, retrying transient failures per the retry policy.
    async fn send_with_retry(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<Response, CliError> {
        let mut attempt: u32 = 1;
        loop {
            let request = builder
                .try_clone()
                .ok_or_else(|| CliError::Generic("request body is not retryable".to_owned()))?;
            match request.send().await {
                Ok(response) => {
                    if is_retryable_status(response.status()) && attempt < self.retry.max_attempts {
                        self.backoff(attempt).await;
                        attempt += 1;
                        continue;
                    }
                    return Ok(response);
                }
                Err(error) => {
                    if is_transient(&error) && attempt < self.retry.max_attempts {
                        self.backoff(attempt).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(CliError::Network(error.to_string()));
                }
            }
        }
    }

    /// Sleeps for the exponential backoff delay of the given attempt.
    async fn backoff(&self, attempt: u32) {
        let factor = 2u32.saturating_pow(attempt - 1);
        tokio::time::sleep(self.retry.base_delay * factor).await;
    }
}

/// Reports whether a status warrants a retry (server errors and rate limiting).
fn is_retryable_status(status: StatusCode) -> bool {
    status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
}

/// Reports whether a transport error is transient and worth retrying.
fn is_transient(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

/// Maps an HTTP response to a typed result, deserializing 2xx JSON bodies.
async fn map_response<T: DeserializeOwned>(response: Response) -> Result<T, CliError> {
    let status = response.status();
    if status.is_success() {
        return response
            .json::<T>()
            .await
            .map_err(|e| CliError::Generic(format!("invalid response body: {e}")));
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::NotAuthenticated);
    }
    let message = response.text().await.unwrap_or_default();
    Err(CliError::Api {
        status: status.as_u16(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{Client, RetryPolicy};
    use crate::error::CliError;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Repo {
        name: String,
    }

    fn fast_client(base_url: &str) -> Client {
        Client::new(base_url, "test-token")
            .expect("build client")
            .with_retry(RetryPolicy {
                max_attempts: 3,
                base_delay: Duration::from_millis(0),
            })
    }

    #[tokio::test]
    async fn get_json_sends_bearer_and_decodes_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/widgets"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Repo {
                name: "widgets".to_owned(),
            }))
            .expect(1)
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let repo: Repo = client.get_json("/repos/acme/widgets").await.expect("ok");
        assert_eq!(repo.name, "widgets");
    }

    #[tokio::test]
    async fn unauthorized_maps_to_not_authenticated() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let err = client.get_json::<Repo>("/repos").await.expect_err("err");
        assert!(matches!(err, CliError::NotAuthenticated));
    }

    #[tokio::test]
    async fn not_found_maps_to_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let err = client.get_json::<Repo>("/repos/x").await.expect_err("err");
        match err {
            CliError::Api { status, message } => {
                assert_eq!(status, 404);
                assert_eq!(message, "missing");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retries_server_errors_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .with_priority(1)
            .expect(2)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Repo {
                name: "ok".to_owned(),
            }))
            .expect(1)
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let repo: Repo = client.get_json("/flaky").await.expect("ok after retries");
        assert_eq!(repo.name, "ok");
    }

    #[tokio::test]
    async fn client_errors_are_not_retried() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bad"))
            .respond_with(ResponseTemplate::new(400).set_body_string("nope"))
            .expect(1)
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let err = client.get_json::<Repo>("/bad").await.expect_err("err");
        assert!(matches!(err, CliError::Api { status: 400, .. }));
    }

    #[tokio::test]
    async fn post_json_sends_body_and_decodes_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/repos"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(201).set_body_json(Repo {
                name: "created".to_owned(),
            }))
            .expect(1)
            .mount(&server)
            .await;

        let client = fast_client(&server.uri());
        let created: Repo = client
            .post_json(
                "/repos",
                &Repo {
                    name: "created".to_owned(),
                },
            )
            .await
            .expect("ok");
        assert_eq!(created.name, "created");
    }
}
