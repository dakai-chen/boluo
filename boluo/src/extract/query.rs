use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

use boluo_core::extract::FromRequest;
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Copy)]
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Query<T> {
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    type Error = ExtractQueryError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let query = req.uri().query().unwrap_or_default();
        serde_urlencoded::from_str::<T>(query)
            .map(|value| Query(value))
            .map_err(ExtractQueryError::FailedToDeserialize)
    }
}

#[derive(Debug, Clone)]
pub struct RawQuery(pub String);

impl Deref for RawQuery {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawQuery {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawQuery {
    #[inline]
    pub fn into_inner(this: Self) -> String {
        this.0
    }
}

impl FromRequest for RawQuery {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(RawQuery(req.uri().query().unwrap_or_default().to_owned()))
    }
}

#[derive(Debug)]
pub enum ExtractQueryError {
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl std::fmt::Display for ExtractQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractQueryError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize query string ({e})")
            }
        }
    }
}

impl std::error::Error for ExtractQueryError {}
