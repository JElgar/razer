use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct StructReceiver {
    ident: syn::Ident,
    data: darling::ast::Data<(), FieldReceiver>,
}

#[derive(Debug, FromField)]
#[darling(attributes(auto_builder))]
struct FieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    // TODO Support method
    // See: https://github.com/serde-rs/serde/blob/846f865de2e94408e0edc6a2c6863c063cd234be/serde_derive/src/internals/attr.rs#L397
    //    : https://serde.rs/attr-default.html

    // #[darling(rename = "default")]
    default: Option<bool>,
}

struct StructInfo {
    ident: syn::Ident,
    display_name: String,
    fields: Vec<FieldInfo>,
}

struct FieldInfo {
    ident: syn::Ident,
    ty: syn::Type,
    display_name: String,
}

enum FieldType {
    String,
    Number,
    Boolean,
}

impl FieldInfo {
    fn field_type(&self) -> FieldType {
        let field_type_ident = match &self.ty {
            syn::Type::Path(syn::TypePath{ path: syn::Path { segments, .. }, .. }) => {
                segments[0].ident.clone()
            }
            _ => panic!("Type not supported {:#?}", self.ty),
        };

        // TODO It would be nice if we could implement a trait for these types
        // I.e. some kind of trait for types which can be intoed to fields or something
        // Note sure if thats possible but could be clean...
        match field_type_ident.to_string().as_str() {
            "String" => FieldType::String,
            "u32" | "i32" => FieldType::Number,
            "bool" => FieldType::Boolean,
            // TODO Better error handling
            _ => panic!("Type not supported {:#?}", self.ty),
        }
    }

    fn field_config(&self) -> proc_macro2::TokenStream {
        let FieldInfo { ident, display_name, .. } = self;
        let field_name = ident.to_string();
        let field_config_values = quote! {
            attribute_name: #field_name.to_string(),
            display_name: #display_name.to_string(),
            default_value: None
        };

        match self.field_type() {
            FieldType::String => quote! {
                razer::FieldConfig::<String> {
                    #field_config_values
                }
            },
            FieldType::Number => quote! {
                razer::FieldConfig::<i64> {
                    #field_config_values
                }
            },
            FieldType::Boolean => quote! {
                razer::FieldConfig::<bool> {
                    #field_config_values
                }
            },
        }
    }

    fn field_definition(&self) -> proc_macro2::TokenStream {
        let field_config = self.field_config();
        match self.field_type() {
            FieldType::String => quote! {
                razer::FieldDef::Text(#field_config)
            },
            FieldType::Number => quote! {
                razer::FieldDef::Number(#field_config)
            },
            FieldType::Boolean => quote! {
                razer::FieldDef::Boolean(#field_config)
            },
        }
    }

    fn field_value(&self) -> proc_macro2::TokenStream {
        let field_config = self.field_config();
        let ident = &self.ident;
        let field_value = quote! {
            razer::FieldWithValue {
                field_config: #field_config,
                value: self.#ident.clone().into()
            }
        };
        match self.field_type() {
            FieldType::String => quote! {
                razer::FieldValue::Text(#field_value)
            },
            FieldType::Number => quote! {
                razer::FieldValue::Number(#field_value)
            },
            FieldType::Boolean => quote! {
                razer::FieldValue::Boolean(#field_value)
            },
        }
    }
}

impl From<StructReceiver> for StructInfo {
    fn from(struct_receiver: StructReceiver) -> Self {
        let StructReceiver { ident, data, .. } = struct_receiver;
        let display_name = ident.to_string();
        StructInfo {
            ident,
            display_name,
            fields: data.take_struct().expect("Only named structs are supported - enforced by darling").fields
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<FieldReceiver> for FieldInfo {
    fn from(field_receiver: FieldReceiver) -> Self {
        let FieldReceiver { ident, ty, default, .. } = field_receiver;
        let display_name = attribute_name_to_display_name(&ident.as_ref().unwrap().to_string());
        FieldInfo {
            ident: ident.expect("Only named fields are supported - enforced by darling"),
            ty,
            display_name,
        }
    }
}

/// TODO Can't run these tests because needs to be public and can't expose from proc macro crate.
/// Would be nice to work this one out...
/// ```
/// assert_eq!(razer::attribute_name_to_display_name("attr_name"), "Attr name");
/// ```
/// ```
/// assert_eq!(razer::attribute_name_to_display_name("hello"), "Hello");
/// ```
/// ```
/// assert_eq!(razer::attribute_name_to_display_name("abc_def_hi"), "Abc def hi");
/// ```
fn attribute_name_to_display_name(name: &str) -> String {
    // Replace underscores with spaces
    let name = name.replace("_", " ");

    // Uppercase the first letter
    let mut name_chars = name.chars();
    match name_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + name_chars.as_str(),
    }
}

#[proc_macro_derive(AdminModel)]
pub fn admin_model_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_info = StructReceiver::from_derive_input(&input);
    match struct_info {
        Err(e) => TokenStream::from(e.write_errors()),
        Ok(struct_info) => impl_admin_model(&struct_info.into()).into(),
    }
}

fn impl_admin_model(input: &StructInfo) -> proc_macro2::TokenStream {
    let admin_base_impl = impl_admin_base(input);

    let struct_ident = input.ident.clone();
    let model_name = input.ident.to_string();
    let field_idents = input
        .fields
        .iter()
        .map(|field| field.ident.clone());

    let field_values = input.fields.iter().map(|field| {
        field.field_value()
    });

    quote! {
        #admin_base_impl

        #[async_trait::async_trait]
        impl razer::AdminModel for #struct_ident {
            fn model_name() -> &'static str {
                #model_name
            }

            fn get_row(&self) -> Vec<String> {
                vec![#( self.#field_idents.clone().to_string() ),*]
            }

            // TODO Replace field values with this...
            fn get_field_values(&self) -> Vec<razer::FieldValue> {
                vec![
                    #(#field_values),*
                ]
            }
        }
    }.into()
}

#[proc_macro_derive(AdminInputModel)]
pub fn admin_input_model_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_info = StructReceiver::from_derive_input(&input);
    match struct_info {
        Err(e) => TokenStream::from(e.write_errors()),
        Ok(struct_info) => impl_admin_input_model(&struct_info.into()).into(),
    }
}

fn impl_admin_input_model(input: &StructInfo) -> proc_macro2::TokenStream {
    let struct_ident = input.ident.clone();

    let admin_base_impl = impl_admin_base(input);

    quote! {
        #admin_base_impl

        #[async_trait::async_trait]
        impl razer::AdminInputModel for #struct_ident {}
    }.into()
}

fn impl_admin_base(input: &StructInfo) -> proc_macro2::TokenStream {
    let struct_ident = input.ident.clone();
    let field_defs = input.fields.iter().map(|field| field.field_definition());

    quote! {
        #[async_trait::async_trait]
        impl razer::AdminModelBase for #struct_ident {
            fn get_field_definitions() -> Vec<razer::FieldDef> {
                vec![
                    #(#field_defs),*
                ]
            }
        }
    }
}
