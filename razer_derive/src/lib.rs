use proc_macro::TokenStream;
use quote::quote;

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
    let field_names: Vec<String> = field_idents.iter().map(|ident| ident.to_string()).collect();

    let field_types = fields.iter().map(|field| match &field.ty {
        syn::Type::Path(syn::TypePath{ path: syn::Path { segments, .. }, .. }) => segments[0].ident.clone(),
        _ => panic!("Type not supported {:#?}", field.ty),
    }).map(
        |field_type_ident| match field_type_ident.to_string().as_str() {
            "String" => quote! {
                "text"
            },
            "u32" => quote! {
                "number"
            },
            type_name => panic!("Unsupported field type {:#?}", type_name),
        }
    );

    quote! {
        impl razer::TableDataType for #struct_ident {
            fn get_headers() -> Vec<razer::TableHeaderData> {
                vec![#( #field_names ),*]
            }

            fn get_row(&self) -> Vec<razer::TableCellData> {
                vec![#( self.#field_idents.clone().to_string() ),*]
            }
        }

        #[async_trait::async_trait]
        impl razer::AdminModel for #struct_ident {
            fn get_field_definitions() -> Vec<razer::FieldDef> {
                vec![
                    #(
                        razer::FieldDef {
                            name: #field_names.to_string(),
                            value_type: #field_types.to_string(),
                            // TODO Potentially add a getter method here and remove table nonsense
                        }
                    ),*
                ]
            }

            fn model_name() -> String {
                #model_name.to_string()
            }
        }
    }.into()
}
