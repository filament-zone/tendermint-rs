//! This module defines the `Authorization` type for
//! authorizing a HTTP or WebSocket RPC client using
//! HTTP Basic authentication.

use alloc::borrow::ToOwned as _;
use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use subtle_encoding::base64;
use url::Url;

use crate::Error;

/// An HTTP authorization.
///
/// Currently only HTTP Basic authentication is supported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authorization {
    Basic(String),
    Bearer(String),
}

impl fmt::Display for Authorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic(cred) => write!(f, "Basic {cred}"),
            Self::Bearer(token) => write!(f, "Bearer {token}"),
        }
    }
}

impl FromStr for Authorization {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(auth) = input.strip_prefix("Basic ") {
            return Ok(Self::Basic(auth.to_owned()));
        } else if let Some(auth) = input.strip_prefix("Bearer ") {
            return Ok(Self::Bearer(auth.to_owned()));
        }

        Err(Error::invalid_authorization())
    }
}

/// Extract the authorization, if any, from the authority part of the given URI.
///
/// This authorization can then be supplied to the RPC server via
/// the `Authorization` HTTP header.
pub fn authorize(url: &Url) -> Option<Authorization> {
    let authority = url.authority();

    if let Some((userpass, _)) = authority.split_once('@') {
        let bytes = base64::encode(userpass);
        let credentials = String::from_utf8_lossy(bytes.as_slice());
        Some(Authorization::Basic(credentials.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_auth_absent() {
        let uri = "http://example.com".parse().unwrap();
        assert_eq!(authorize(&uri), None);
    }

    #[test]
    fn extract_auth_username_only() {
        let uri = "http://toto@example.com".parse().unwrap();
        let base64 = "dG90bw==".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }

    #[test]
    fn extract_auth_username_password() {
        let uri = "http://toto:tata@example.com".parse().unwrap();
        let base64 = "dG90bzp0YXRh".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }
}
