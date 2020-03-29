use crate::error::Error;
use crate::model::{Request, Response};
#[cfg(test)]
use mockito;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde_json::json;

#[cfg(not(test))]
fn jqdata_url() -> String {
    String::from("https://dataapi.joinquant.com/apis")
}

#[cfg(test)]
fn jqdata_url() -> String {
    mockito::server_url()
}

pub struct JqdataClient {
    token: String,
}

/// retrieve token with given credential
fn get_token(mob: &str, pwd: &str, reuse: bool) -> Result<String, Error> {
    let method = if reuse {
        "get_current_token"
    } else {
        "get_token"
    };
    let token_req = json!({
        "method": method,
        "mob": mob,
        "pwd": pwd,
    });
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&jqdata_url())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(token_req.to_string())
        .send()?;
    let token: String = response.text()?;
    if token.starts_with("error") {
        return Err(Error::Server(token));
    }
    Ok(token)
}

impl JqdataClient {
    pub fn with_credential(mob: &str, pwd: &str) -> Result<Self, Error> {
        let token = get_token(mob, pwd, true)?;
        Ok(JqdataClient { token })
    }

    pub fn with_token(token: &str) -> Result<Self, Error> {
        Ok(JqdataClient {
            token: token.to_string(),
        })
    }

    pub fn execute<C: Request + Response>(&self, command: C) -> Result<C::Output, Error> {
        let req_body = command.request(&self.token)?;
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&jqdata_url())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(req_body)
            .send()?;
        let output = command.response(response)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use mockito::mock;

    #[test]
    fn test_error_response() {
        let response_body = "error: invalid token";
        let _m = mock("POST", "/")
            .with_status(200)
            .with_body(response_body)
            .create();

        let client = JqdataClient::with_token("abc").unwrap();
        let response = client.execute(GetAllSecurities {
            code: SecurityKind::Stock,
            date: None,
        });
        assert!(response.is_err());
    }

    #[test]
    fn test_get_all_securities() {
        let response_body = {
            let mut s = String::from("code,display_name,name,start_date,end_date,type\n");
            s.push_str("000001.XSHE,平安银行,PAYH,1991-04-03,2200-01-01,stock\n");
            s.push_str("000002.XSHE,万科A,WKA,1991-01-29,2200-01-01,stock\n");
            s
        };
        let _m = mock("POST", "/")
            .with_status(200)
            .with_body(&response_body)
            .create();

        let client = JqdataClient::with_token("abc").unwrap();
        let ss = client
            .execute(GetAllSecurities {
                code: SecurityKind::Stock,
                date: None,
            })
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
    }
}
