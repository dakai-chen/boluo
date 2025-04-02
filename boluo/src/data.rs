//! 通用数据类型。

use std::ops::{Deref, DerefMut};

/// JSON 提取器和响应。
///
/// # 提取
///
/// 当用作提取器时，[`Json`] 可以将请求体反序列化为实现 [`serde::de::DeserializeOwned`] 的类型。
///
/// ```
/// use boluo::data::Json;
///
/// #[derive(serde::Deserialize)]
/// struct CreateUser {
///     username: String,
///     password: String,
/// }
///
/// #[boluo::route("/users", method = "POST")]
/// async fn create_user(Json(user): Json<CreateUser>) {
///     // 创建用户。
/// }
/// ```
///
/// # 响应
///
/// 当用作响应时，[`Json`] 可以将实现 [`serde::Serialize`] 的类型序列化为 JSON，
/// 并设置响应标头 `Content-Type: application/json`。
///
/// ```
/// use boluo::data::Json;
/// use boluo::extract::Path;
///
/// #[derive(serde::Serialize)]
/// struct User {
///     id: String,
///     username: String,
/// }
///
/// #[boluo::route("/users/{id}", method = "GET")]
/// async fn get_user(Path(id): Path<String>) -> Json<User> {
///     let user = find_user(&id).await;
///     Json(user)
/// }
///
/// async fn find_user(id: &str) -> User {
///     todo!()
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Json<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

/// 表单提取器和响应。
///
/// # 提取
///
/// 当用作提取器时，[`Form`] 可以将请求中的表单数据反序列化为实现
/// [`serde::de::DeserializeOwned`] 的类型。
///
/// 如果请求具有 `GET` 或 `HEAD` 方法，则会从查询字符串中读取表单数据（与 [`Query`] 相同），
/// 如果请求具有不同的方法，则会从请求体中读取表单数据。
///
/// ```
/// use boluo::data::Form;
///
/// #[derive(serde::Deserialize)]
/// struct CreateUser {
///     username: String,
///     password: String,
/// }
///
/// #[boluo::route("/users", method = "POST")]
/// async fn create_user(Form(user): Form<CreateUser>) {
///     // 创建用户。
/// }
/// ```
///
/// # 响应
///
/// 当用作响应时，[`Form`] 可以将实现 [`serde::Serialize`] 的类型序列化为表单，
/// 并设置响应标头 `Content-Type: application/x-www-form-urlencoded`。
///
/// ```
/// use boluo::data::Form;
/// use boluo::extract::Path;
///
/// #[derive(serde::Serialize)]
/// struct User {
///     id: String,
///     username: String,
/// }
///
/// #[boluo::route("/users/{id}", method = "GET")]
/// async fn get_user(Path(id): Path<String>) -> Form<User> {
///     let user = find_user(&id).await;
///     Form(user)
/// }
///
/// async fn find_user(id: &str) -> User {
///     todo!()
/// }
/// ```
///
/// [`Query`]: crate::extract::Query
#[derive(Debug, Clone, Copy)]
pub struct Form<T>(pub T);

impl<T> Deref for Form<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Form<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

/// 扩展提取器和响应。
///
/// # 提取
///
/// 这通常用于在处理程序之间共享状态。
///
/// ```
/// use std::sync::Arc;
///
/// use boluo::data::Extension;
/// use boluo::route::Router;
/// use boluo::service::ServiceExt;
///
/// // 在整个应用程序中使用的一些共享状态。
/// struct State {
///     // ...
/// }
///
/// #[boluo::route("/")]
/// async fn handler(Extension(state): Extension<Arc<State>>) {
///     // ...
/// }
///
/// let state = Arc::new(State { /* ... */ });
///
/// Router::new().mount(handler)
///     // 添加中间件，将状态插入到所有传入请求的扩展中。
///     .with(Extension(state));
/// ```
///
/// # 响应
///
/// 响应扩展可以用于与中间件共享状态。
///
/// ```
/// use boluo::data::Extension;
/// use boluo::response::IntoResponse;
///
/// #[derive(Clone)]
/// struct Foo(&'static str);
///
/// async fn handler() -> impl IntoResponse {
///     (Extension(Foo("foo")), "Hello, World!")
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Extension<T>(pub T);

impl<T> Deref for Extension<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Extension<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Extension<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}
