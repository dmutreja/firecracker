use std::result;

use futures::sync::oneshot;
use hyper::{Response, StatusCode};

use http_service::{json_fault_message, json_response};
use request::{GenerateResponse, ParsedRequest, SyncRequest};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum APILoggerLevel {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct APILoggerDescription {
    pub log_fifo: String,
    pub metrics_fifo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<APILoggerLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_level: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_log_origin: Option<bool>,
}

#[derive(Debug)]
pub enum APILoggerError {
    InitializationFailure(String),
}

impl GenerateResponse for APILoggerError {
    fn generate_response(&self) -> Response {
        use self::APILoggerError::*;
        match *self {
            InitializationFailure(ref e) => json_response(
                StatusCode::BadRequest,
                json_fault_message(format!{"Cannot initialize logging system! {}", e}),
            ),
        }
    }
}

impl APILoggerDescription {
    pub fn into_parsed_request(self) -> result::Result<ParsedRequest, String> {
        let (sender, receiver) = oneshot::channel();
        Ok(ParsedRequest::Sync(
            SyncRequest::PutLogger(self, sender),
            receiver,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_response_logger_error() {
        assert_eq!(
            APILoggerError::InitializationFailure("Could not initialize log system".to_string())
                .generate_response()
                .status(),
            StatusCode::BadRequest
        );
        assert!(
            format!(
                "{:?}",
                APILoggerError::InitializationFailure(
                    "Could not initialize log system".to_string()
                )
            ).contains("InitializationFailure")
        );
    }

    #[test]
    fn test_into_parsed_request() {
        let desc = APILoggerDescription {
            log_fifo: String::from("log"),
            metrics_fifo: String::from("metrics"),
            level: None,
            show_level: None,
            show_log_origin: None,
        };
        format!("{:?}", desc);
        assert!(&desc.clone().into_parsed_request().is_ok());
        let (sender, receiver) = oneshot::channel();
        assert!(
            &desc
                .clone()
                .into_parsed_request()
                .eq(&Ok(ParsedRequest::Sync(
                    SyncRequest::PutLogger(desc, sender),
                    receiver
                )))
        );
    }
}