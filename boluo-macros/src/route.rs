use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Ident, ItemFn, LitStr, Token, Visibility};

pub(crate) fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as RouteAttr);

    let item_fn = match syn::parse::<ItemFn>(item.clone()) {
        Ok(item_fn) => item_fn,
        Err(e) => return input_and_compile_error(item, e),
    };

    match Route::new(attr, item_fn) {
        Ok(route) => route.into_token_stream().into(),
        Err(e) => input_and_compile_error(item, e),
    }
}

struct PathAttr(String);

impl Parse for PathAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<LitStr>().map(|s| s.value()).map(Self)
    }
}

impl ToTokens for PathAttr {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        stream.append(LitStr::new(self.0.as_str(), Span::call_site()).token());
    }
}

#[derive(PartialEq, Eq, Hash)]
struct MethodAttr(String);

impl MethodAttr {
    fn parse_more(input: ParseStream<'_>) -> syn::Result<Vec<Self>> {
        let content;
        let _bracket_token = syn::bracketed!(content in input);
        let methods = content.parse_terminated(MethodAttr::parse_one, Token![,])?;
        Ok(methods.into_iter().collect())
    }

    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<LitStr>().and_then(MethodAttr::try_from)
    }

    fn parse(input: ParseStream<'_>) -> syn::Result<Vec<Self>> {
        if MethodAttr::parse_more(&input.fork()).is_ok() {
            return MethodAttr::parse_more(input);
        }
        MethodAttr::parse_one(input).map(|v| vec![v])
    }
}

impl ToTokens for MethodAttr {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        stream.append(LitStr::new(self.0.as_str(), Span::call_site()).token());
    }
}

impl TryFrom<LitStr> for MethodAttr {
    type Error = Error;

    fn try_from(value: LitStr) -> Result<Self, Self::Error> {
        let method = value.value();
        if method.is_empty() {
            Err(Error::new_spanned(value, "invalid HTTP method"))
        } else {
            Ok(Self(method))
        }
    }
}

struct CratePath(syn::Path);

impl CratePath {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<LitStr>()?.parse::<syn::Path>().map(Self)
    }
}

impl ToTokens for CratePath {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        self.0.to_tokens(stream);
    }
}

struct RouteAttr {
    path: PathAttr,
    methods: Vec<MethodAttr>,
    crate_path: Option<CratePath>,
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = PathAttr::parse(input).map_err(|_| {
            Error::new(
                Span::call_site(),
                r#"invalid route definition, expected #[route("<path>", ...)]"#,
            )
        })?;

        let mut methods = None;
        let mut crate_path = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let ident = Ident::parse_any(input)?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "method" => {
                    if methods.is_some() {
                        return Err(Error::new_spanned(
                            &ident,
                            format!("duplicate attribute `{ident}`"),
                        ));
                    }
                    methods = Some(MethodAttr::parse(input)?);
                }
                "crate" => {
                    if crate_path.is_some() {
                        return Err(Error::new_spanned(
                            &ident,
                            format!("duplicate attribute `{ident}`"),
                        ));
                    }
                    crate_path = Some(CratePath::parse(input)?);
                }
                _ => {
                    return Err(Error::new_spanned(
                        &ident,
                        format!("illegal attribute `{ident}`"),
                    ));
                }
            }
        }

        Ok(Self {
            path,
            methods: methods.unwrap_or_default(),
            crate_path,
        })
    }
}

struct Route {
    item_fn: ItemFn,
    vis: Visibility,
    name: Ident,
    attr: RouteAttr,
    docs: Vec<Attribute>,
}

impl Route {
    fn new(attr: RouteAttr, item_fn: ItemFn) -> syn::Result<Self> {
        let vis = item_fn.vis.clone();
        let name = item_fn.sig.ident.clone();

        let docs = item_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect();

        Ok(Self {
            item_fn,
            vis,
            name,
            attr,
            docs,
        })
    }
}

impl ToTokens for Route {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            item_fn,
            vis,
            name,
            attr,
            docs,
        } = self;

        let RouteAttr {
            path,
            methods,
            crate_path,
        } = attr;

        let crate_path = if let Some(name) = crate_path {
            quote!(#name)
        } else {
            quote!(::boluo)
        };

        let methods = methods.iter();

        let handler_service = quote! {
            #crate_path::service::Service<#crate_path::request::Request,
                Response = #crate_path::response::Response,
                Error = #crate_path::BoxError,
            >
        };

        let stream = quote! {
            #(#docs)*
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            #vis struct #name;

            impl #crate_path::service::Service<#crate_path::request::Request> for #name {
                type Response = #crate_path::response::Response;
                type Error = #crate_path::BoxError;

                async fn call(
                    &self,
                    request: #crate_path::request::Request,
                ) -> ::std::result::Result<Self::Response, Self::Error> {
                    #item_fn

                    fn assert_service<S>(service: S) -> impl #handler_service
                    where
                        S: #handler_service,
                    {
                        service
                    }

                    let service = #crate_path::handler::handler_fn(#name);
                    let service = assert_service(service);

                    #crate_path::service::Service::call(&service, request).await
                }
            }

            impl ::std::convert::Into<#crate_path::route::Route<#name>> for #name {
                fn into(self) -> #crate_path::route::Route<#name> {
                    let method_route = #crate_path::route::any(#name)
                        #(.add(#crate_path::http::Method::try_from(#methods).unwrap()))*;
                    #crate_path::route::Route::new(#path, method_route)
                }
            }
        };

        tokens.extend(stream);
    }
}

fn input_and_compile_error(mut item: TokenStream, err: Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}
