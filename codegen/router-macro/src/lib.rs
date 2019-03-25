#![recursion_limit = "128"]

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use heck::{CamelCase, SnakeCase};
use proc_macro2::Span;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::Error;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::MetaList;
use syn::NestedMeta;
use syn::{parse::Result, Ident};

type TokenStream = proc_macro::TokenStream;

trait IdentExt {
    fn append(&self, string: &str) -> Ident;
    fn camel_case(&self) -> Ident;
    fn snake_case(&self) -> Ident;
    fn to_upper(&self) -> Ident;
}

impl IdentExt for syn::Ident {
    fn append(&self, string: &str) -> Ident {
        Ident::new(&format!("{}{}", self, string), self.span())
    }

    fn camel_case(&self) -> Ident {
        Ident::new(&self.to_string().to_camel_case(), self.span())
    }

    fn snake_case(&self) -> Ident {
        Ident::new(&self.to_string().to_snake_case(), self.span())
    }

    fn to_upper(&self) -> Ident {
        Ident::new(&self.to_string().to_uppercase(), self.span())
    }
}

/// A container type for the parsed tokens.  
pub(crate) struct Input {
    pub parsed_struct: Struct,
}

/// A generic composable builder trait.  
pub(crate) trait Builder<'i> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream>;
}

/// Attributes on the struct itself.  
#[derive(Debug)]
pub(crate) struct Attrs;

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let _attrs = input.call(Attribute::parse_outer)?;

        Ok(Attrs {})
    }
}

/// Attributes on the fields within the struct.  
#[derive(Debug)]
pub(crate) struct FieldAttr {
    pub method: Ident,
    pub path: LitStr,
}

/// A field in the struct
#[derive(Debug)]
pub(crate) struct Field {
    attr: FieldAttr,
    pub name: Ident,
    pub ty: Ident,
}

impl Field {
    fn error(span: Span) -> Result<FieldAttr> {
        let message = "expected one of:\n\t#[get(\"...\")]\n\t#[post(\"...\")]\n\t#[put(\"...\")]\n\t#[delete(\"...\")]";
        Err(Error::new(span, message))
    }

    fn method(&self) -> proc_macro2::TokenStream {
        let method = &self.attr.method.to_upper();
        quote!(#method)
    }

    fn path(&self) -> proc_macro2::TokenStream {
        let path = &self.attr.path;
        quote!(#path)
    }

    fn parse_attr(input: &ParseStream) -> Result<FieldAttr> {
        let mut span = Span::call_site();
        let meta = input
            .call(Attribute::parse_outer)?
            .iter()
            .map(|attr| {
                span = attr.bracket_token.span;
                Ok(attr.parse_meta()?)
            })
            .collect::<Result<Vec<Meta>>>();

        match meta {
            Err(_) => {
                Self::error(span)
            }
            Ok(meta) => {
                if meta.len() == 0 {
                    return Self::error(span);
                }
                let method = meta[0].name();
                let nested = match &meta[0] {
                    Meta::List(MetaList { nested, .. }) if nested.first().is_some() => {
                        nested.first().unwrap().into_value()
                    }
                    _ => {
                        let message = "Unsupported route specification";
                        return Err(Error::new(span, message));
                    }
                };

                match nested {
                    NestedMeta::Literal(Lit::Str(lit_str)) => Ok(FieldAttr {
                        method,
                        path: lit_str.clone(),
                    }),
                    _ => {
                        let message = "expected \"path\"";
                        return Err(Error::new(span, message));
                    }
                }
            }
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr = Self::parse_attr(&input)?;
        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty: Ident = input.parse()?;

        Ok(Field { attr, name, ty })
    }
}

/// The parsed struct.  
#[derive(Debug)]
pub(crate) struct Struct {
    pub attrs: Attrs,
    pub ident: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attrs::parse)?;
        let content;
        let _: Token![struct] = input.parse()?;
        let ident = input.parse()?;
        let _ = braced!(content in input);
        let fields = content.parse_terminated(Field::parse)?;
        Ok(Struct {
            attrs,
            ident,
            fields,
        })
    }
}

struct Routes;

impl<'i> Builder<'i> for Routes {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        input.parsed_struct.fields.iter().for_each(|field| {
            let ty = &field.ty;
            let path = field.path();
            let method = field.method();

            fields.push(quote! {
                #path if parts.method == http::Method::#method => {

                    boxed!(Request::<<#ty as Route>::Body>::parse(req)
                           .and_then(|request| #ty::process_request(request)))
                }
            });
        });
        Ok(quote!(#(#fields)*))
    }
}

///
/// ### User Defined Routes
///
/// ```
/// #[routes]
/// struct Router {
///     #[get("/")]
///     index: Index,
///     #[post("/profile")]
///     profile: Profile,
/// }
/// ```
///
/// ### Generated Router
///
/// ```
/// pub struct Router;
///
/// impl Route for Router {
///     type Body = h2::RecvStream;
///     type Future = RouteF<Response>;
///
///     fn handle_request(req: Request<Self::Body>) -> Self::Future {
///         let parts = &req.parts();
///         match parts.uri.path() {
///             "/" if parts.method == http::Method::GET => {
///                 boxed!(Request::<<Index as Route>::Body>::parse(req)
///                     .and_then(|request| Index::process_request(request)))
///             }
///             "/profile" if parts.method == http::Method::POST => {
///                 boxed!(Request::<<Profile as Route>::Body>::parse(req)
///                     .and_then(|request| Profile::process_request(request)))
///             }
///             _ => {
///                 let res = Response::new()
///                     .status(http::StatusCode::NOT_FOUND)
///                     .content_type("application/json")
///                     .body(json_bytes_ok!(json!({ "message": "not found" })));
///
///                 boxed!(OkFut(res))
///             }
///         }
///     }
/// }
/// ```
///

#[proc_macro_attribute]
pub fn routes(_: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(input as Struct);
    let parsed = Input { parsed_struct };
    let name = &parsed.parsed_struct.ident;
    let routes = Routes.build(&parsed).unwrap();

    let generated = quote_spanned! {Span::call_site()=>
        pub struct #name;

        impl Route for #name {
            type Body = h2::RecvStream;
            type Future = RouteF<Response>;

            fn handle_request(req: Request<Self::Body>) -> Self::Future {
                let parts = &req.parts();
                match parts.uri.path() {
                    #routes
                    _ => {
                        let res = Response::new()
                            .status(http::StatusCode::NOT_FOUND)
                            .content_type("application/json")
                            .body(json_bytes_ok!(json!({ "message": "not found" })));

                        boxed!(OkFut(res))
                    }
                }
            }
        }
    };

    generated.into()
}
