use std::io::Cursor;

use thiserror::Error;
use axum::extract::rejection::QueryRejection;
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, header::{self}};
use validator::ValidationErrors;
use sea_orm::error::DbErr;
use quick_xml::Error as QXmlError;
use quick_xml::{events::{Event, BytesStart}, writer::Writer, escape::escape};

// use crate::app::errors;

#[derive(Debug, Error)]
pub enum ProfileServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
    #[error(transparent)]
    SeaOrmDbError(#[from] DbErr),
    #[error(transparent)]
    QuickXmlError(#[from] QXmlError),
    #[error("realm '{0}' is not configured")]
    RealmNotConfigured(String),
    #[error("realm '{0}' digest '{1}' incorrect")]
    RealmDigestIncorrect(String, String),
    #[error("player '{1}' [hash:{0}] sid {2} != {3}")]
    PlayerSidMismatch(i64, String, i64, i64),
    #[error("player '{1}' [hash:{0}, sid:{2}] rid '{3}' incorrect")]
    PlayerRidIncorrect(i64, String, i64, String),
}

impl ProfileServerError {
    pub fn as_xml(self) -> String {
        let mut error_data_xml_writer = Writer::new(Cursor::new(Vec::new()));
        let mut data_element_start = BytesStart::new("data");
        // push ok="1" for all atm, todo: make more specific via RE of rwr_server to discover ok codes
        data_element_start.push_attribute(("ok", "0"));
        let issue = match self {
            ProfileServerError::ValidationError(err) => err.to_string(),
            ProfileServerError::AxumQueryRejection(err) => err.to_string(),
            ProfileServerError::SeaOrmDbError(err) => err.to_string(),
            ProfileServerError::QuickXmlError(err) => err.to_string(),
            ProfileServerError::RealmNotConfigured(realm_name) =>
                format!("realm '{realm_name}' not configured"),
            ProfileServerError::RealmDigestIncorrect(realm_name, realm_digest) =>
                format!("realm '{realm_name}' digest '{realm_digest}' incorrect"),
            ProfileServerError::PlayerSidMismatch(hash, username, given_sid, _expected_sid) =>
                format!("player '{username}' [{hash}] sid '{given_sid}' does not match existing sid"),
            ProfileServerError::PlayerRidIncorrect(hash, username, _sid, given_rid) =>
                format!("player '{username}' [{hash}] rid '{given_rid}' incorrect"),
        };
        // escape the issue :D
        let escaped_issue = escape(&issue).to_string();
        data_element_start.push_attribute(("issue", escaped_issue.as_str()));
        match error_data_xml_writer.write_event(Event::Empty(data_element_start)) {
            Ok(_) => String::from_utf8(error_data_xml_writer.into_inner().into_inner()).unwrap(),
            Err(err) => {
                tracing::error!("failed to write xml data event for ProfileServerError response: {}", err.to_string());
                String::from("<data ok=\"0\"")
            }
        }
    }   
}

impl IntoResponse for ProfileServerError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.to_string());
        // let headers  = [(header::CONTENT_TYPE, "application/xml")];
        let headers  = [(header::CONTENT_TYPE, "text/xml")];
        match self {
            ProfileServerError::ValidationError(_) => (StatusCode::BAD_REQUEST, headers, self.as_xml()),
            ProfileServerError::AxumQueryRejection(_) => (StatusCode::BAD_REQUEST, headers, self.as_xml()),
            ProfileServerError::SeaOrmDbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, headers, self.as_xml()),
            ProfileServerError::QuickXmlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, headers, self.as_xml()),
            ProfileServerError::RealmNotConfigured(_) => (StatusCode::BAD_REQUEST, headers, self.as_xml()),
            ProfileServerError::RealmDigestIncorrect(_, _) => (StatusCode::UNAUTHORIZED, headers, self.as_xml()),
            ProfileServerError::PlayerSidMismatch(_, _, _, _) => (StatusCode::UNAUTHORIZED, headers, self.as_xml()),
            ProfileServerError::PlayerRidIncorrect(_, _, _, _) => (StatusCode::UNAUTHORIZED, headers, self.as_xml()),
        }
        .into_response()
    }
}