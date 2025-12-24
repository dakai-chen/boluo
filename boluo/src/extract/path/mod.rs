mod de;

use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

use boluo_core::extract::FromRequest;
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

use crate::route::PathParams;

/// 获取原始路径参数的提取器，不对路径参数进行解析。
///
/// # 例子
///
/// ```
/// use boluo::extract::RawPathParams;
///
/// #[boluo::route("/classes/{class_id}/students/{stdnt_id}", method = "GET")]
/// async fn handler(RawPathParams(params): RawPathParams) {
///     for (k, v) in params {
///         println!("{k}: {v}");
///     }
/// }
/// ```
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
    /// 得到内部的值。
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
                .map(|PathParams(params)| params.clone())
                .unwrap_or_default(),
        ))
    }
}

/// 用于获取路径参数的提取器。
///
/// `T` 需要实现 [`serde::de::DeserializeOwned`]。
///
/// # 例子
///
/// ```
/// use std::collections::HashMap;
///
/// use boluo::extract::Path;
///
/// #[derive(serde::Deserialize)]
/// struct Params {
///     class_id: i32,
///     stdnt_id: i32,
/// }
///
/// // 使用结构体提取路径参数，结构体的字段名需要和路径参数名相同。
/// #[boluo::route("/classes/{class_id}/students/{stdnt_id}", method = "GET")]
/// async fn using_struct(Path(Params { class_id, stdnt_id }): Path<Params>) {
///     // ...
/// }
///
/// // 可以使用 `HashMap` 提取所有路径参数。
/// #[boluo::route("/classes/{class_id}/students/{stdnt_id}", method = "GET")]
/// async fn using_hashmap(Path(hashmap): Path<HashMap<String, String>>) {
///     // ...
/// }
///
/// // 使用元组提取路径参数，元组的大小和路径参数数量必须相同，路径参数将按照顺序解析到元组中。
/// #[boluo::route("/classes/{class_id}/students/{stdnt_id}", method = "GET")]
/// async fn using_tuple(Path((class_id, stdnt_id)): Path<(i32, i32)>) {
///     // ...
/// }
///
/// // 可以使用 `Vec` 提取所有路径参数，路径参数将按照顺序解析到数组中。
/// #[boluo::route("/classes/{class_id}/students/{stdnt_id}", method = "GET")]
/// async fn using_vec(Path(vec): Path<Vec<String>>) {
///     // ...
/// }
///
/// // 如果路径参数只有一个，使用元组提取时，可以省略元组。
/// #[boluo::route("/classes/{class_id}", method = "GET")]
/// async fn only_one(Path(class_id): Path<i32>) {
///     // ...
/// }
/// ```
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
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    type Error = PathError;

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

/// 路径参数提取错误。
#[derive(Debug, Clone)]
pub enum PathError {
    /// 参数数量不正确。
    WrongNumberOfParameters {
        /// 实际的参数数量。
        got: usize,
        /// 预期的参数数量。
        expected: usize,
    },
    /// 尝试反序列化为不受支持的键类型。
    UnsupportedKeyType {
        /// 键类型名。
        name: &'static str,
    },
    /// 尝试反序列化为不受支持的值类型。
    UnsupportedValueType {
        /// 值类型名。
        name: &'static str,
    },
    /// 解析错误。
    ParseError(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::WrongNumberOfParameters { got, expected } => write!(
                f,
                "wrong number of path arguments. expected {expected} but got {got}"
            ),
            PathError::UnsupportedKeyType { name } => {
                write!(f, "unsupported key type `{name}`")
            }
            PathError::UnsupportedValueType { name } => {
                write!(f, "unsupported value type `{name}`")
            }
            PathError::ParseError(msg) => {
                write!(f, "failed to parse path arguments ({msg})")
            }
        }
    }
}

impl std::error::Error for PathError {}

impl From<de::PathDeError> for PathError {
    fn from(error: de::PathDeError) -> Self {
        match error {
            de::PathDeError::WrongNumberOfParameters { got, expected } => {
                PathError::WrongNumberOfParameters { got, expected }
            }
            de::PathDeError::UnsupportedKeyType { name } => PathError::UnsupportedKeyType { name },
            de::PathDeError::UnsupportedValueType { name } => {
                PathError::UnsupportedValueType { name }
            }
            _ => PathError::ParseError(error.to_string()),
        }
    }
}
