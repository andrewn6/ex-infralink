use std::fmt;

use serde::Serialize;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Serialize)]
pub struct ServiceError {
    pub message: String,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Service error: {}", self.message)
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        ServiceError{
            message: error.to_string(),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(self)
    }
}