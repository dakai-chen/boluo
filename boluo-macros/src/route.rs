use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
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
    fn parse_array(input: ParseStream<'_>) -> syn::Result<Vec<Self>> {
        let content;
        let _bracket_token = syn::bracketed!(content in input);

        let methods = content.parse_terminated(MethodAttr::parse_str, Token![,])?;
        Ok(methods.into_iter().collect())
    }

    fn parse_str(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<LitStr>().and_then(MethodAttr::try_from)
    }

    fn parse(input: ParseStream<'_>) -> syn::Result<Vec<Self>> {
        let name = input.parse::<Ident>()?;

        if name != "method" {
            return Err(Error::new_spanned(
                &name,
                format!("illegal attribute `{name}`"),
            ));
        }

        input.parse::<Token![=]>()?;

        if MethodAttr::parse_array(&input.fork()).is_ok() {
            return MethodAttr::parse_array(input);
        }

        MethodAttr::parse_str(input).map(|v| vec![v])
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

struct RouteAttr {
    path: PathAttr,
    methods: Vec<MethodAttr>,
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = PathAttr::parse(input).map_err(|_| {
            Error::new(
                Span::call_site(),
                r#"invalid route definition, expected #[route("<path>")]"#,
            )
        })?;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let methods = if input.is_empty() {
            vec![]
        } else {
            MethodAttr::parse(input)?
        };

        Ok(Self { path, methods })
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

        let RouteAttr { path, methods } = attr;

        let methods = methods.iter();

        let stream = quote! {
            #(#docs)*
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            #vis struct #name;

            impl ::boluo::service::Service<::boluo::request::Request> for #name {
                type Response = ::boluo::response::Response;
                type Error = ::boluo::BoxError;

                async fn call(
                    &self,
                    req: ::boluo::request::Request,
                ) -> ::std::result::Result<Self::Response, Self::Error> {
                    #item_fn

                    fn assert_service<S>(
                        service: S,
                    ) -> impl ::boluo::service::Service<
                        ::boluo::request::Request,
                        Response = ::boluo::response::Response,
                        Error = ::boluo::BoxError,
                    >
                    where
                        S: ::boluo::service::Service<
                            ::boluo::request::Request,
                            Response = ::boluo::response::Response,
                            Error = ::boluo::BoxError,
                        >,
                    {
                        service
                    }

                    let service = ::boluo::handler::handler_fn(#name);
                    let service = assert_service(service);

                    ::boluo::service::Service::call(&service, req).await
                }
            }

            impl ::std::convert::Into<::boluo::route::Route<#name>> for #name {
                fn into(self) -> ::boluo::route::Route<#name> {
                    let method_route = ::boluo::route::any(#name)
                        #(.add(::boluo::http::Method::try_from(#methods).unwrap()))*;
                    ::boluo::route::Route::new(#path, method_route)
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
