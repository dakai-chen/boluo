use boluo_core::http::Extensions;
use matchit::Params;

use super::router::PRIVATE_TAIL_PARAM;

/// 路由器提取的路径参数。
#[derive(Debug, Clone)]
pub(crate) struct PathParams(pub(crate) Vec<(String, String)>);

impl PathParams {
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    fn push(&mut self, param: (String, String)) {
        self.0.push(param)
    }

    #[inline]
    fn extend(&mut self, other: Self) {
        self.0.extend(other.0)
    }
}

pub(super) fn prase_path_params(params: Params) -> (PathParams, Option<String>) {
    let mut path_params = PathParams::with_capacity(params.len());
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
        path_params.extend(params);
    } else {
        extensions.insert(params);
    }
}
