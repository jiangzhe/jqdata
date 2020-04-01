pub use jqdata_model::*;

use crate::Error;
use crate::{Request, HasMethod, BodyConsumer};
#[cfg(test)]
use mockito;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde_json::json;
use std::sync::Arc;
use futures_util::lock::Mutex;
use serde::{Serialize, Deserialize};

/// provide static jqdata API url
/// 
/// use #[cfg(test)] to switch this address with mockito address
#[cfg(not(test))]
fn jqdata_url() -> String {
    String::from("https://dataapi.joinquant.com/apis")
}

#[cfg(test)]
fn jqdata_url() -> String {
    mockito::server_url()
}

/// JqdataClient
/// 
/// async client for jqdata API
#[derive(Clone)]
pub struct JqdataClient {
    inner: Arc<Mutex<Arc<SharedClient>>>,
}

impl JqdataClient {

    /// Create new client with given credential
    /// 
    /// This method will try to refresh token using the given
    /// credential, causing itself to be async
    pub async fn with_credential(mob: String, pwd: String) -> Result<Self> {
        let mut shared_cli = SharedClient{
            credential: Some(ClientCredential{ mob, pwd }),
            token: String::new(),
        };
        shared_cli.refresh_token().await?;
        Ok(JqdataClient{
            inner: Arc::new(Mutex::new(Arc::new(shared_cli))),
        })
    }

    /// Execute request in async context, 
    /// 
    /// Aync context should be tokio 0.2, because the reqwest crate 
    /// depends on it
    pub async fn execute<T, C>(&self, command: C) -> Result<T> 
    where 
        T: for<'de> Deserialize<'de>,
        T: Serialize,
        C: HasMethod + BodyConsumer<T> + Serialize,
    {
        let shared_cli = {
            let cli_ref = &*self.inner.lock().await;
            Arc::clone(cli_ref)
        };
        let req_body = Request::new(shared_cli.token.to_owned(), command);
        let body = serde_json::to_string(&req_body)?;
        let client = reqwest::Client::new();
        let response = client
            .post(&crate::jqdata_url())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(body)
            .send()
            .await
            .map_err(|e| Error::Client(e.to_string()))?
            .text()
            .await
            .map_err(|e| Error::Client(e.to_string()))?;
        let output = <C as BodyConsumer<_>>::consume_body(response.as_bytes())?;
        Ok(output)
    }
}

struct SharedClient {
    credential: Option<ClientCredential>,
    token: String,
}

impl SharedClient {
    async fn refresh_token(&mut self) -> Result<()> {
        if self.credential.is_none() {
            return Err(Error::Client("credential not available to refresh token".to_owned()));
        }

        let token_req = json!({
            "method": "get_current_token",
            "mob": self.credential.as_ref().unwrap().mob,
            "pwd": self.credential.as_ref().unwrap().pwd,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&jqdata_url())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(token_req.to_string())
            .send()
            .await
            .map_err(|e| Error::Client(e.to_string()))?;
        let token = response.text().await.map_err(|e| Error::Client(e.to_string()))?;
        if token.starts_with("error") {
            return Err(Error::Server(token));
        }
        self.token = token;
        Ok(())
    }
}

/// internal struct to hold client credential
struct ClientCredential {
    mob: String,
    pwd: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use crate::{GetAllSecurities, SecurityKind, Security};

    #[tokio::test]
    async fn test_get_all_securities() -> std::io::Result<()> {
        let response_body = {
            let mut s = String::from("code,display_name,name,start_date,end_date,type\n");
            s.push_str("000001.XSHE,平安银行,PAYH,1991-04-03,2200-01-01,stock\n");
            s.push_str("000002.XSHE,万科A,WKA,1991-01-29,2200-01-01,stock\n");
            s
        };
        let _mock_api = mock("POST", "/")
            .with_status(200)
            .with_body(&response_body)
            .create();
        
        let client = {
            let _mock_token = mock("POST", "/")
                .with_status(200)
                .with_body("abc")
                .create();
            JqdataClient::with_credential("10000".to_owned(), "pass".to_owned()).await.unwrap()
        };
        let ss = client
            .execute(GetAllSecurities {
                code: SecurityKind::Stock,
                date: None,
            })
            .await
            .unwrap();
        assert_eq!(
            vec![
                Security {
                    code: "000001.XSHE".to_string(),
                    display_name: "平安银行".to_string(),
                    name: "PAYH".to_string(),
                    start_date: "1991-04-03".to_string(),
                    end_date: "2200-01-01".to_string(),
                    kind: SecurityKind::Stock,
                    parent: None,
                },
                Security {
                    code: "000002.XSHE".to_string(),
                    display_name: "万科A".to_string(),
                    name: "WKA".to_string(),
                    start_date: "1991-01-29".to_string(),
                    end_date: "2200-01-01".to_string(),
                    kind: SecurityKind::Stock,
                    parent: None,
                }
            ],
            ss
        );
        Ok(())
    }
}
