use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Attribute, Meta, Lit, MetaList, Token, Expr};
use syn::punctuated::Punctuated;

/// Attribute macro for configuring admin panel options
#[proc_macro_derive(AdminPanel, attributes(admin))]
pub fn derive_admin_panel(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let attrs = parse_admin_attributes(&input.attrs);

    let resource_name = if attrs.name.is_empty() {
        name.to_string()
    } else {
        attrs.name.clone()
    };
    let resource_path = if attrs.path.is_empty() {
        resource_name.to_lowercase()
    } else {
        attrs.path.clone()
    };
    let list_fields = &attrs.list_fields;
    let create_fields = &attrs.create_fields;
    let edit_fields = &attrs.edit_fields;
    let search_fields = &attrs.search_fields;
    let filter_fields = &attrs.filter_fields;

    let expanded = quote! {
        #[async_trait::async_trait]
        impl AdminModel for #name {
            fn admin_name() -> &'static str {
                #resource_name
            }

            fn admin_path() -> &'static str {
                #resource_path
            }

            fn list_fields() -> Vec<String> {
                const FIELDS: &[&str] = &[#(#list_fields),*];
                FIELDS.iter().map(|s| s.to_string()).collect()
            }

            fn create_fields() -> Vec<AdminField> {
                vec![]
            }

            fn edit_fields() -> Vec<AdminField> {
                vec![]
            }

            fn search_fields() -> Vec<String> {
                const FIELDS: &[&str] = &[#(#search_fields),*];
                FIELDS.iter().map(|s| s.to_string()).collect()
            }

            fn filter_fields() -> Vec<String> {
                const FIELDS: &[&str] = &[#(#filter_fields),*];
                FIELDS.iter().map(|s| s.to_string()).collect()
            }

            async fn find_by_id(id: i32) -> Result<Self, AdminError> {
                Self::find_by_id(id).await
            }

            async fn find_all(params: ListParams) -> Result<Vec<Self>, AdminError> {
                Self::find_all(params).await
            }

            async fn create(data: Self) -> Result<Self, AdminError> {
                Self::create(data).await
            }

            async fn update(id: i32, data: Self) -> Result<Self, AdminError> {
                Self::update(id, data).await
            }

            async fn delete(id: i32) -> Result<(), AdminError> {
                Self::delete(id).await
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Helper struct to store parsed admin attributes
#[derive(Default)]
struct AdminAttributes {
    name: String,
    path: String,
    list_fields: Vec<String>,
    create_fields: Vec<String>,
    edit_fields: Vec<String>,
    search_fields: Vec<String>,
    filter_fields: Vec<String>,
}

/// Parse admin attributes from a struct's attributes
fn parse_admin_attributes(attrs: &[Attribute]) -> AdminAttributes {
    let mut admin_attrs = AdminAttributes::default();
    
    for attr in attrs {
        if !attr.path().is_ident("admin") {
            continue;
        }
        
        let meta = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .expect("Failed to parse admin attributes");
            
        for meta_item in meta {
            match meta_item {
                Meta::NameValue(nv) if nv.path.is_ident("name") => {
                    if let Expr::Lit(expr_lit) = nv.value {
                        if let Lit::Str(s) = expr_lit.lit {
                            admin_attrs.name = s.value();
                        }
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("path") => {
                    if let Expr::Lit(expr_lit) = nv.value {
                        if let Lit::Str(s) = expr_lit.lit {
                            admin_attrs.path = s.value();
                        }
                    }
                }
                Meta::List(list) if list.path.is_ident("list_fields") => {
                    admin_attrs.list_fields = parse_string_array(&list);
                }
                Meta::List(list) if list.path.is_ident("create_fields") => {
                    admin_attrs.create_fields = parse_string_array(&list);
                }
                Meta::List(list) if list.path.is_ident("edit_fields") => {
                    admin_attrs.edit_fields = parse_string_array(&list);
                }
                Meta::List(list) if list.path.is_ident("search_fields") => {
                    admin_attrs.search_fields = parse_string_array(&list);
                }
                Meta::List(list) if list.path.is_ident("filter_fields") => {
                    admin_attrs.filter_fields = parse_string_array(&list);
                }
                _ => {}
            }
        }
    }
    
    admin_attrs
}

fn parse_string_array(list: &MetaList) -> Vec<String> {
    let nested: Punctuated<Lit, Token![,]> = list.parse_args_with(Punctuated::parse_terminated)
        .expect("Failed to parse string array");
    
    println!("Debug - Parsing string array: {:?}", list);
    println!("Debug - Nested tokens: {:?}", nested);
        
    let result = nested.into_iter()
        .map(|lit| {
            let val = match lit {
                Lit::Str(s) => s.value(),
                _ => {
                    let token_str = lit.to_token_stream().to_string();
                    token_str.trim_matches('"').to_string()
                }
            };
            println!("Debug - Parsed value: {:?}", val);
            val
        })
        .collect::<Vec<_>>();
    
    println!("Debug - Final result: {:?}", result);
    result
}
