use std::fmt;

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, FieldsNamed, Ident, Meta, Token};

struct AdminResourceDeriveArgs {
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct CompileError {
    msg: String,
    span: Option<Span>,
}

impl CompileError {
    fn new(message: &str) -> Self {
        Self {
            msg: message.to_string(),
            span: None,
        }
    }
}

impl std::error::Error for CompileError {}
impl fmt::Display for CompileError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&self.msg)
    }
}

/// ```
/// assert_eq!(razer::attribute_name_to_display_name("attr_name"), "Attr name");
/// ```
/// ```
/// assert_eq!(razer::attribute_name_to_display_name("hello"), "Hello");
/// ```
/// ```
/// assert_eq!(razer::attribute_name_to_display_name("abc_def_hi"), "Abc def hi");
/// ```
fn field_id_to_display_name(name: &str) -> String {
    // Replace underscores with spaces
    let name = name.replace("_", " ");

    // Uppercase the first letter
    let mut name_chars = name.chars();
    match name_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + name_chars.as_str(),
    }
}

#[proc_macro_derive(AdminResource, attributes(admin))]
pub fn derive_admin_resource(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    // TODO No unwrap
    derive_admin_resource_impl(input).unwrap()
}

struct AdminFieldData {
    name: syn::Ident,
    ty: syn::Type,
    is_readonly: bool,
}

enum FieldType {
    String,
    Number,
    Boolean,
}

impl FieldType {
    fn from_syn_type(ty: &syn::Type) -> Result<Self, CompileError> {
        let field_type_ident = match &ty {
            syn::Type::Path(syn::TypePath {
                path: syn::Path { segments, .. },
                ..
            }) => Ok(segments[0].ident.clone()),
            _ => Err(CompileError::new("Field type not supported")),
        }?;

        match field_type_ident.to_string().as_str() {
            "String" => Ok(FieldType::String),
            "u32" | "i32" => Ok(FieldType::Number),
            "bool" => Ok(FieldType::Boolean),
            // TODO Better error handling
            _ => Err(CompileError::new("Field type not supported")),
        }
    }
}

fn derive_admin_resource_impl(input: DeriveInput) -> Result<TokenStream, CompileError> {
    let struct_fields = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(FieldsNamed { named: fields, .. }),
            ..
        }) => Ok(fields.iter().map(|field| {
            let admin_attr = field
                .attrs
                .iter()
                .find(|it| it.path().get_ident().is_some_and(|it| it == "admin"));

            let args = admin_attr.map(|it| {
                // TODO Handle error
                it.parse_args_with(<Punctuated<Meta, Token![,]>>::parse_terminated)
                    .unwrap()
            });

            let mut is_readonly = false;

            if let Some(args) = args {
                args.iter().for_each(|attr| {
                    dbg!(attr);
                });

                args.iter().for_each(|arg| {
                    match arg {
                        Meta::Path(path) if path.get_ident().is_some_and(|it| it == "readonly") => {
                            is_readonly = true;
                        },
                        _ => todo!()
                    }
                });
            }

            AdminFieldData {
                name: field.ident.clone().expect("Named fields must have idents"),
                ty: field.ty.clone(),
                is_readonly,
            }
        })),
        _ => Err(CompileError::new("Aa")),
    }?;

    let struct_ident = input.ident;

    let field_config_defs = struct_fields.clone().map(|field| {
        let field_id = field.name.to_string();
        let field_name = field_id_to_display_name(&field_id.to_string());
        let is_readonly = field.is_readonly;

        // TODO Don't unwrap
        let function_ident = match FieldType::from_syn_type(&field.ty).unwrap() {
            FieldType::String => quote! { create_text_config },
            FieldType::Number => quote! { create_number_config },
            FieldType::Boolean => quote! { create_boolean_config },
        };

        quote! {
            razer_core::FieldConfig::#function_ident(#field_id.to_string(), #field_name.to_string(), #is_readonly)
        }
    });

    let admin_attr = input
        .attrs
        .iter()
        .find(|it| it.path().get_ident().is_some_and(|it| it == "admin"));

    match admin_attr {
        Some(admin_attr) => {
            let args = admin_attr
                .parse_args_with(<Punctuated<Meta, Token![,]>>::parse_terminated)
                .map_err(|e| {
                    todo!()
                    // CompileError::no_file_info(
                    //     format_args!("unable to parse template arguments: {e}"),
                    //     Some(attr.path().span()),
                    // )
                });
        }
        None => todo!(),
    }

    let field_configs_struct_ident: Ident = Ident::new(&format!("{}FieldConfigs", struct_ident), proc_macro2::Span::call_site());
    let field_configs_struct_fields = struct_fields.clone().map(|field| {
        let field_ident = field.name;
        quote! {
            #field_ident: razer_core::FieldConfig
        }
    });

    let field_configs_struct_into = {
        let assignments = struct_fields.map(|field| {
            let field_ident = field.name;
            quote! {
                self.#field_ident
            }
        });

        quote! {
            impl Into<Vec<razer_core::FieldConfig>> for #field_configs_struct_ident {
                fn into(self) -> Vec<razer_core::FieldConfig> {
                    vec![
                        #(#assignments),*
                    ]
                }
            }
        }
    };

    Ok(quote! {
        impl #struct_ident {
            fn default_field_configs() -> Vec<razer_core::FieldConfig> {
                vec![
                    #(#field_config_defs),*
                ]
            }
        }

        struct #field_configs_struct_ident {
            #(#field_configs_struct_fields),*
        }

        #field_configs_struct_into
    }
    .into())
}
