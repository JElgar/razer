use proc_macro::TokenStream;
use quote::quote;

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
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse_macro_input!(input as syn::ItemStruct);

    let struct_ident = &ast.ident;
    let model_name = &ast.ident.to_string();

    // TODO It would be nice if we could implement a trait for these types
    // I.e. some kind of trait for types which can be intoed to fields or something
    // Note sure if thats possible but could be clean...

    let fields = match &ast.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => panic!("Only named fields are supported"),
    };

    let field_idents: Vec<syn::Ident> = fields.iter().map(|field| field.ident.clone().unwrap()).collect();

    let admin_base_impl = impl_admin_base(&ast);

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
        }
    }.into()
}

#[proc_macro_derive(AdminInputModel)]
pub fn admin_input_model_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemStruct);

    let struct_ident = &ast.ident;

    let admin_base_impl = impl_admin_base(&ast);

    quote! {
        #admin_base_impl

        #[async_trait::async_trait]
        impl razer::AdminInputModel for #struct_ident {}
    }.into()
}

fn impl_admin_base(_struct: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let struct_ident = &_struct.ident;

    // TODO It would be nice if we could implement a trait for these types
    // I.e. some kind of trait for types which can be intoed to fields or something
    // Note sure if thats possible but could be clean...
    let fields = match &_struct.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => panic!("Only named fields are supported"),
    };

    let field_idents: Vec<syn::Ident> = fields.iter().map(|field| field.ident.clone().unwrap()).collect();
    let field_names: Vec<String> = field_idents.iter().map(|ident| ident.to_string()).collect();
    let field_display_names: Vec<String> = field_idents.iter().map(|ident| attribute_name_to_display_name(&ident.to_string())).collect();

    let field_types = fields.iter().map(|field| match &field.ty {
        syn::Type::Path(syn::TypePath{ path: syn::Path { segments, .. }, .. }) => segments[0].ident.clone(),
        _ => panic!("Type not supported {:#?}", field.ty),
    }).map(
        |field_type_ident| match field_type_ident.to_string().as_str() {
            "String" => quote! {
                "text"
            },
            "u32" | "i32" => quote! {
                "number"
            },
            "bool" => quote! {
                "boolean"
            },
            // TODO Use better errors so macro users can debug
            type_name => panic!("Unsupported field type {:#?}", type_name),
        }
    );

    quote! {
        #[async_trait::async_trait]
        impl razer::AdminModelBase for #struct_ident {
            fn get_field_definitions() -> Vec<razer::FieldDef> {
                vec![
                    #(
                        razer::FieldDef {
                            attribute_name: #field_names.to_string(),
                            attribute_type: #field_types.to_string(),
                            display_name: #field_display_names.to_string(),
                            // TODO Potentially add a getter method here and remove table nonsense
                        }
                    ),*
                ]
            }
        }
    }
}
