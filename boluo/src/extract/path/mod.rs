mod de;

use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

use boluo_core::extract::FromRequest;
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

use crate::route::PathParams;

#[derive(Debug, Clone)]
pub struct RawPathParams(pub Vec<(String, String)>);

impl Deref for RawPathParams {
    type Target = Vec<(String, String)>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawPathParams {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawPathParams {
    #[inline]
    pub fn into_inner(this: Self) -> Vec<(String, String)> {
        this.0
    }
}

impl FromRequest for RawPathParams {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(RawPathParams(
            req.extensions()
                .get::<PathParams>()
                .map(|PathParams(ref params)| params.clone())
                .unwrap_or_default(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Path<T>(pub T);

impl<T> Deref for Path<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Path<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Path<T> {
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    type Error = PathExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let path_params = match req.extensions().get::<PathParams>() {
            Some(PathParams(path_params)) => &path_params[..],
            None => &[],
        };
        Ok(Path(T::deserialize(de::PathDeserializer::new(
            path_params,
        ))?))
    }
}

#[derive(Debug)]
pub enum PathExtractError {
    /// 参数数量不正确
    WrongNumberOfParameters { got: usize, expected: usize },

    /// 尝试反序列化为不受支持的键类型
    UnsupportedKeyType { name: &'static str },

    /// 尝试反序列化为不受支持的值类型
    UnsupportedValueType { name: &'static str },

    /// 解析错误
    ParseError(String),
}

impl std::fmt::Display for PathExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathExtractError::WrongNumberOfParameters { got, expected } => write!(
                f,
                "wrong number of path arguments. expected {expected} but got {got}"
            ),
            PathExtractError::UnsupportedKeyType { name } => {
                write!(f, "unsupported key type `{name}`")
            }
            PathExtractError::UnsupportedValueType { name } => {
                write!(f, "unsupported value type `{name}`")
            }
            PathExtractError::ParseError(msg) => {
                write!(f, "failed to parse path arguments ({msg})")
            }
        }
    }
}

impl std::error::Error for PathExtractError {}

impl From<de::PathDeserializationError> for PathExtractError {
    fn from(error: de::PathDeserializationError) -> Self {
        match error {
            de::PathDeserializationError::WrongNumberOfParameters { got, expected } => {
                PathExtractError::WrongNumberOfParameters { got, expected }
            }
            de::PathDeserializationError::UnsupportedKeyType { name } => {
                PathExtractError::UnsupportedKeyType { name }
            }
            de::PathDeserializationError::UnsupportedValueType { name } => {
                PathExtractError::UnsupportedValueType { name }
            }
            _ => PathExtractError::ParseError(error.to_string()),
        }
    }
}
