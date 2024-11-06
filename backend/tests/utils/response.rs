use http::StatusCode;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct TestResponse {
    response: reqwest::Response,
}

#[allow(unused)]
impl TestResponse {
    pub(super) fn new(response: reqwest::Response) -> Self {
        TestResponse { response }
    }

    pub async fn json<T: DeserializeOwned>(self) -> T {
        self.response
            .json()
            .await
            .expect("failed to deserialize to json")
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }
}
