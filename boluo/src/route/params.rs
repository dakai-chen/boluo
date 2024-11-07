use std::ops::{Deref, DerefMut};

use boluo_core::http::Extensions;
use matchit::Params;

use super::router::PRIVATE_TAIL_PARAM;

/// 路由器提取的路径参数。
#[derive(Default, Debug, Clone)]
pub struct PathParams(pub Vec<(String, String)>);

impl Deref for PathParams {
    type Target = Vec<(String, String)>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathParams {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(super) fn prase_path_params(params: Params<'_, '_>) -> (PathParams, Option<String>) {
    let mut path_params = PathParams(Vec::with_capacity(params.len()));
    let mut tail_params = None;

    for (name, value) in params.iter() {
        if name == PRIVATE_TAIL_PARAM {
            tail_params = Some(if value.starts_with('/') {
                value.to_owned()
            } else {
                format!("/{value}")
            });
        } else {
            path_params.push((name.to_owned(), value.to_owned()));
        }
    }

    (path_params, tail_params)
}

pub(super) fn insert_path_params(extensions: &mut Extensions, params: PathParams) {
    if let Some(path_params) = extensions.get_mut::<PathParams>() {
        path_params.extend(params.0);
    } else {
        extensions.insert(params);
    }
}
