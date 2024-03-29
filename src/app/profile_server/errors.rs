use std::io::Cursor;
use std::net::IpAddr;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use axum::extract::rejection::{BytesRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use quick_xml::Error as QXmlError;
use quick_xml::{
    escape::escape,
    events::{BytesStart, Event},
    writer::Writer,
};
use sea_orm::error::DbErr;
use thiserror::Error;
use validator::ValidationErrors;

use super::util::HEADERS;

#[derive(Debug, Error)]
pub enum ProfileServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    AxumBytesRejection(#[from] BytesRejection),
    #[error(transparent)]
    SeaOrmDbError(#[from] DbErr),
    #[error(transparent)]
    QuickXmlError(#[from] QXmlError),
    #[error(transparent)]
    QuickXmlDeserializationFailed(#[from] quick_xml::DeError),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("ip address '{0}' not allowed to get/set")]
    ClientAddressNotAllowed(IpAddr),
    #[error("realm '{0}' is not configured")]
    RealmNotConfigured(String),
    #[error("sid '{0}' not allowed by config")]
    SidNotAllowed(i64),
    #[error("sid '{0}' blocked by config")]
    SidBlocked(i64),
    #[error("realm '{0}' digest '{1}' incorrect")]
    RealmDigestIncorrect(String, String),
    #[error("player '{1}' [hash:{0}] sid {2} mismatch")]
    PlayerSidMismatch(i64, String, i64, i64),
    #[error("player '{1}' [hash:{0}, sid:{2}] rid '{3}' incorrect")]
    PlayerRidIncorrect(i64, String, i64, String),
    #[error("player '{1}' [hash:{0}, sid:{2}] not found in db")]
    PlayerNotFound(i64, String, i64),
}

impl ProfileServerError {
    pub fn to_xml_string(&self) -> String {
        let mut error_data_xml_writer = Writer::new(Cursor::new(Vec::new()));
        let mut data_element_start = BytesStart::new("data");
        // push ok="1" for all atm, todo: make more specific via RE of rwr_server to discover ok codes
        data_element_start.push_attribute(("ok", "0"));
        let msg = match self {
            ProfileServerError::ValidationError(err) => err.to_string(),
            ProfileServerError::AxumQueryRejection(err) => err.to_string(),
            ProfileServerError::AxumBytesRejection(err) => err.to_string(),
            ProfileServerError::SeaOrmDbError(err) => err.to_string(),
            ProfileServerError::QuickXmlError(err) => err.to_string(),
            ProfileServerError::QuickXmlDeserializationFailed(err) => err.to_string(),
            ProfileServerError::Utf8Error(err) => err.to_string(),
            ProfileServerError::FromUtf8Error(err) => err.to_string(),
            ProfileServerError::SerdeJsonError(err) => err.to_string(),
            ProfileServerError::ClientAddressNotAllowed(_) => self.to_string(),
            ProfileServerError::RealmNotConfigured(_) => self.to_string(),
            ProfileServerError::SidNotAllowed(_) => self.to_string(),
            ProfileServerError::SidBlocked(_) => self.to_string(),
            ProfileServerError::RealmDigestIncorrect(_, _) => self.to_string(),
            ProfileServerError::PlayerSidMismatch(_, _, _, _) => self.to_string(),
            ProfileServerError::PlayerRidIncorrect(_, _, _, _) => self.to_string(),
            ProfileServerError::PlayerNotFound(_, _, _) => self.to_string(),
        };
        // escape the message :D
        let escaped_msg = escape(&msg).to_string();
        data_element_start.push_attribute(("msg", escaped_msg.as_str()));
        match error_data_xml_writer.write_event(Event::Empty(data_element_start)) {
            Ok(_) => String::from_utf8(error_data_xml_writer.into_inner().into_inner()).unwrap(),
            Err(err) => {
                tracing::error!(
                    "failed to write xml data event for ProfileServerError response: {}",
                    err.to_string()
                );
                String::from("<data ok=\"0\"")
            }
        }
    }
}

impl IntoResponse for ProfileServerError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.to_string());
        match self {
            ProfileServerError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
            ProfileServerError::AxumQueryRejection(_) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
            ProfileServerError::AxumBytesRejection(_) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
            ProfileServerError::SeaOrmDbError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HEADERS,
                self.to_xml_string(),
            ),
            ProfileServerError::QuickXmlError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HEADERS,
                self.to_xml_string(),
            ),
            ProfileServerError::QuickXmlDeserializationFailed(_) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
            ProfileServerError::Utf8Error(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HEADERS,
                self.to_xml_string(),
            ),
            ProfileServerError::FromUtf8Error(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HEADERS,
                self.to_xml_string(),
            ),
            ProfileServerError::SerdeJsonError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HEADERS,
                self.to_xml_string(),
            ),
            ProfileServerError::ClientAddressNotAllowed(_) => {
                (StatusCode::UNAUTHORIZED, HEADERS, self.to_xml_string())
            }
            ProfileServerError::RealmNotConfigured(_) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
            ProfileServerError::SidNotAllowed(_) => {
                (StatusCode::FORBIDDEN, HEADERS, self.to_xml_string())
            }
            ProfileServerError::SidBlocked(_) => {
                (StatusCode::FORBIDDEN, HEADERS, self.to_xml_string())
            }
            ProfileServerError::RealmDigestIncorrect(_, _) => {
                (StatusCode::UNAUTHORIZED, HEADERS, self.to_xml_string())
            }
            ProfileServerError::PlayerSidMismatch(_, _, _, _) => {
                (StatusCode::UNAUTHORIZED, HEADERS, self.to_xml_string())
            }
            ProfileServerError::PlayerRidIncorrect(_, _, _, _) => {
                (StatusCode::UNAUTHORIZED, HEADERS, self.to_xml_string())
            }
            ProfileServerError::PlayerNotFound(_, _, _) => {
                (StatusCode::BAD_REQUEST, HEADERS, self.to_xml_string())
            }
        }
        .into_response()
    }
}
