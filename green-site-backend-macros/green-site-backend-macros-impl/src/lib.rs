use std::{convert::Infallible, str::FromStr};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, AttrStyle,
    Attribute, Ident, ItemStruct, LitStr, Meta, NestedMeta, TypePath,
};

extern crate proc_macro;
#[derive(PartialEq, Eq, Copy, Clone)]
enum AllowedTypes {
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    Unknown,
}

impl FromStr for AllowedTypes {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "String" => Ok(AllowedTypes::String),
            "u8" => Ok(AllowedTypes::U8),
            "u16" => Ok(AllowedTypes::U16),
            "u32" => Ok(AllowedTypes::U32),
            "u64" => Ok(AllowedTypes::U64),
            "u128" => Ok(AllowedTypes::U128),
            "usize" => Ok(AllowedTypes::USize),
            "i8" => Ok(AllowedTypes::I8),
            "i16" => Ok(AllowedTypes::I16),
            "i32" => Ok(AllowedTypes::I32),
            "i64" => Ok(AllowedTypes::I64),
            "i128" => Ok(AllowedTypes::I128),
            "isize" => Ok(AllowedTypes::ISize),
            _ => Ok(AllowedTypes::Unknown),
        }
    }
}

impl ToTokens for AllowedTypes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let to_append = match self {
            AllowedTypes::String => quote!(std::string::String),
            AllowedTypes::U8 => quote!(core::primitive::u8),
            AllowedTypes::U16 => quote!(core::primitive::u16),
            AllowedTypes::U32 => quote!(core::primitive::u32),
            AllowedTypes::U64 => quote!(core::primitive::u64),
            AllowedTypes::U128 => quote!(core::primitive::u128),
            AllowedTypes::USize => quote!(core::primitive::usize),
            AllowedTypes::I8 => quote!(core::primitive::i8),
            AllowedTypes::I16 => quote!(core::primitive::i16),
            AllowedTypes::I32 => quote!(core::primitive::i32),
            AllowedTypes::I64 => quote!(core::primitive::i64),
            AllowedTypes::I128 => quote!(core::primitive::i128),
            AllowedTypes::ISize => quote!(core::primitive::isize),
            AllowedTypes::Unknown => panic!("Type unknown should never be converted to tokens."),
        };

        tokens.append_all(to_append);
    }
}

struct EnvVar {
    var_name: LitStr,
    var_type: AllowedTypes,
    field_ident: Ident,
}

impl EnvVar {
    fn new(var: &Attribute, args: Punctuated<NestedMeta, Comma>) -> syn::Result<Self> {
        if args.len() < 2 || args.len() > 3 {
            return Err(syn::Error::new(
                var.span(),
                format!("Expected a list of 2 or 3 arguments. Got {}. See the documentation for more info.", args.len()),
            ));
        }

        let var_name = syn::parse2::<LitStr>(args[0].to_token_stream())?;
        let var_type_path = syn::parse2::<TypePath>(args[1].to_token_stream())?;

        let field_ident = if args.len() == 3 {
            syn::parse2::<Ident>(args[2].to_token_stream())?
        } else {
            syn::parse_str(var_name.value().to_lowercase().as_str()).map_err(|_| syn::Error::new(
                var_name.span(),
                "Environment variable name specified isn't a valid identifier. Its lowercase form must be a valid Rust identifier.",
            ))?
        };
        let type_ident = var_type_path.path.get_ident();
        let var_type = type_ident
            .and_then(|ident| ident.to_string().parse::<AllowedTypes>().ok())
            .filter(|&unqual| unqual != AllowedTypes::Unknown);

        if let Some(var_type) = var_type {
            Ok(EnvVar {
                var_name,
                var_type,
                field_ident,
            })
        } else {
            return Err(syn::Error::new(
                var_type_path.span(),
                format!("Bad environment variable type. Must be a String or integer type."),
            ));
        }
    }
}

const PARSE_TO_NUM: &'static str = "parse_var_to_num";
const GET_VAR_STR: &'static str = "get_var";

/// Makes a struct have fields that are environment variables where each new environment variable can be specified
/// with an ``#[env_var]`` attribute, which takes the environment variable's name, the type to parse it into, and an optional
/// field name for the environment variable in the struct. If no field name is specified, it will use the lowercased form of the
/// environment variable as the field name. See example for more info.
///
/// Example:
///
/// ```
/// #[env_vars]
/// #[env_var("MY_FIRST_ENV_VAR", String)] // Accepts ``String`` or any integer type to parse environment variable into.
/// #[env_var("MY_SECOND_ENV_VAR", u16, second)] // Accepts optional third argument for struct field name. By default, it's the environment variable lowercased.
/// pub struct MyEnvVars;
/// ```
///
/// This is equivalent to
///
/// ```
/// #[derive(Debug, Clone)]
/// pub struct MyEnvVars {
///     pub my_first_env_var: String,
///     pub second: u16
/// }
///
/// impl MyEnvVars {
///     fn get_var(var: &'static str) -> Result<String, EnvVarParseError> {
///         env::var(var).map_err(|e| EnvVarParseError::EnvVarError(var, e))
///     }
///
///     fn parse_var_to_num<T: FromStr<Err = ParseIntError>>(var: &'static str) -> Result<T, EnvVarParseError> {
///        let var_str = Self::get_var(var)?;
///        var_str
///            .parse()
///            .map_err(|e| EnvVarParseError::NumConversionError(var, var_str, e))
///     }
///
///     pub fn new() -> Result<MyEnvVars, EnvVarParseError> {
///         Ok(MyEnvVars {
///             my_first_env_var: get_var("MY_FIRST_ENV_VAR")?,
///             second: parse_var_to_num("MY_SECOND_ENV_VAR"?,
///         })
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn env_vars(args: TokenStream, input_stream: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(Span::call_site(), "Expected no arguments on macro.")
            .to_compile_error()
            .into();
    }

    let input_struct = parse_macro_input!(input_stream as ItemStruct);
    let env_vars_iter = input_struct
        .attrs
        .into_iter()
        .filter(|attr| matches!(attr.style, AttrStyle::Outer) && attr.path.is_ident("env_var"));
    let mut struct_fields = Vec::new();
    let mut struct_field_assignments = Vec::new();
    let get_var_fn = Ident::new(GET_VAR_STR, Span::call_site());
    let parse_to_num_fn = Ident::new(PARSE_TO_NUM, Span::call_site());

    for env_var in env_vars_iter {
        if let Ok(Meta::List(list)) = env_var.parse_meta() {
            let args = list.nested;

            let EnvVar {
                var_name,
                var_type,
                field_ident,
            } = match EnvVar::new(&env_var, args) {
                Ok(parsed) => parsed,
                Err(err) => return TokenStream::from(err.to_compile_error()),
            };

            struct_fields.push(quote!(pub #field_ident: #var_type));

            if var_type == AllowedTypes::String {
                struct_field_assignments.push(quote!(#field_ident: Self::#get_var_fn(#var_name)?));
            } else {
                struct_field_assignments.push(quote!(#field_ident: Self::#parse_to_num_fn(#var_name)?));
            }
        }
    }

    let input_struct_vis = input_struct.vis;
    let input_struct_name = input_struct.ident;
    let new_comment = format!("Generates a new {input_struct_name} where the fields are the environment \
                               variables specified by the macros. If any environment variable couldn't be \
                               retrieved or parsed, an ``EnvVarParseError`` is returned.");

    quote! {
        #[derive(core::fmt::Debug, core::clone::Clone)]
        #input_struct_vis struct #input_struct_name {
            #(#struct_fields),*
        }

        impl #input_struct_name {
            fn #get_var_fn(var: &'static core::primitive::str) -> core::result::Result<std::string::String, green_site_backend_macros::EnvVarParseError> {
                std::env::var(var).map_err(|e| green_site_backend_macros::EnvVarParseError::EnvVarError(var, e))
            }
 
            fn #parse_to_num_fn<T: core::str::FromStr<Err = core::num::ParseIntError>>(var: &'static core::primitive::str) -> core::result::Result<T, green_site_backend_macros::EnvVarParseError> {
               let var_str = Self::#get_var_fn(var)?;
               var_str
                   .parse()
                   .map_err(|e| green_site_backend_macros::EnvVarParseError::NumConversionError(var, var_str, e))
            }

            #[doc = #new_comment]
            pub fn new() -> core::result::Result<Self, green_site_backend_macros::EnvVarParseError> {
                Ok(Self {
                    #(#struct_field_assignments),*
                })
            }
        }
    }.into()
}
