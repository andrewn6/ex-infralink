use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}, Client, Response};
use serde_json::json;
use std::error::Error;