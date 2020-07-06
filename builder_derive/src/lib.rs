#![feature(proc_macro_diagnostic)]

extern crate proc_macro;
extern crate syn;
extern crate quote;

use std::convert::TryFrom;
use std::iter::IntoIterator;

use proc_macro::{Diagnostic, Level, TokenStream};
use syn::Data::Struct;
use syn::{Attribute, DataStruct, Ident, Type};
use quote::{format_ident, quote};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
enum Error {
    ParseError(syn::Error),
    InvalidParameter(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "parse error: {}", &e),
            Self::InvalidParameter(s) => write!(f, "invalid attribute parameter `{}`", &s)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseError(e) => Some(e),
            _ => None,
        }
    }
}

static ALLOWED_KEYWORDS: [&str; 1] = ["ignore"];

fn is_allowed_keyword(keyword: &Ident) -> bool {
    ALLOWED_KEYWORDS.iter().any(|&x| keyword == x)
}

#[derive(Clone, Debug, Default)]
struct FieldSettings {
    ignore: bool,
}

impl<'a> TryFrom<&'a [Attribute]> for FieldSettings {
    type Error = Error;

    fn try_from(attributes: &'a [Attribute]) -> Result<Self, Self::Error> {
        attributes.iter()
            .filter(|&x| {
                match x.path.get_ident() {
                    Some(ident) if ident == "builder" => true,
                    _ => false
                }
            }).map(|x| x.parse_args::<Ident>())
            .try_fold(FieldSettings::default(), |mut acc, new| {
                match new {
                    Ok(ident) if is_allowed_keyword(&ident) => {
                        acc.ignore = acc.ignore || ident == "ignore";
                        Ok(acc)
                    }
                    Ok(ident) => Err(Error::InvalidParameter(ident.to_string())),
                    Err(e) => Err(Error::ParseError(e))
                }
            })
    }
}

#[derive(Clone)]
struct FieldInfo {
    name: Ident,
    typ: Type,
    settings: FieldSettings,
}

fn generate_impl_block<T>(struct_name: &Ident, infos: T) -> TokenStream where T: IntoIterator<Item=FieldInfo> {
    let methods = infos.into_iter()
        .filter(|field: &FieldInfo| !field.settings.ignore)
        .map(|field: FieldInfo| {
            let name = field.name;
            let typ = field.typ;
            let method_name = format_ident!("set_{}", &name);
            quote!(
                pub fn #method_name(mut self, value: #typ) -> Self {
                    self.#name = value;
                    self
                }
            )
        }).fold(quote!(), |acc, new| quote!(#acc #new));

    quote!(
        impl #struct_name {
            #methods
        }
    ).into()
}

fn collect_fields(struct_info: DataStruct) -> Result<Vec<FieldInfo>, Error> {
    struct_info.fields.into_iter().map(|field| {
        FieldSettings::try_from(field.attrs.as_ref()).map(|settings| {
            FieldInfo {
                name: field.ident.unwrap(),
                typ: field.ty,
                settings,
            }
        })
    }).collect::<Result<Vec<_>, _>>()
}

fn builder_impl(ast: syn::DeriveInput) -> TokenStream {
    if let Struct(s) = ast.data {
        let struct_name = ast.ident;
        let fields = collect_fields(s);

        match fields {
            Ok(infos) => generate_impl_block(&struct_name, infos),
            Err(e) => {
                Diagnostic::new(Level::Error, format!("cannot generate the builder pattern for `{}`: {}", &struct_name, &e)).emit();
                TokenStream::new()
            }
        }
    } else {
        Diagnostic::new(Level::Error, "`Builder` can only be derived for structs").emit();
        TokenStream::new()
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    builder_impl(syn::parse(input).unwrap())
}
