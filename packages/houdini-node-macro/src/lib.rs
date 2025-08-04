use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, parse_macro_input};

#[proc_macro_derive(EntityFromAttribute, attributes(attr))]
pub fn derive_entity_from_attribute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_entity_from_attribute(&input)
}

#[proc_macro_derive(EntityIntoAttribute, attributes(attr))]
pub fn derive_entity_into_attribute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_entity_into_attribute(&input)
}

fn impl_entity_from_attribute(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_loads: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let attr_name = get_field_name(field);
            quote! {
                let #field_name = houdini_node::load_from_attr(attrs.remove(#attr_name).unwrap())?;
            }
        })
        .collect();

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_construction = quote! {
        houdini_node::itertools::izip!(#(#field_names),*).map(|(#(#field_names),*)| Self { #(#field_names),* })
    };

    let generated = quote! {
        impl houdini_node::EntityFromAttribute for #name {
            fn from_attr(
                mut attrs: std::collections::HashMap<String, houdini_node::RawAttribute>,
            ) -> houdini_node::Result<impl Iterator<Item = Self>> {
                #(#field_loads)*
                Ok(#field_construction)
            }
        }
    };
    generated.into()
}

fn impl_entity_into_attribute(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let attr_names: Vec<_> = fields.iter().map(get_field_name).collect();

    let vec_types = fields.iter().map(|_| quote! { Vec<_>});

    let multiunzip_pattern = quote! { (#(#field_names,)*) };
    let multiunzip_types = quote! { #(#vec_types,)* };

    let entity_map = quote! { |entity| (#(entity.#field_names,)*) };

    let hashmap_entries: Vec<_> = field_names
        .iter()
        .zip(attr_names.iter())
        .map(|(name, name_str)| {
            quote! { (#name_str, houdini_node::generate_to_attr(#name)) }
        })
        .collect();

    let generated = quote! {
        impl houdini_node::EntityIntoAttribute for #name {
            fn into_attr(entities: Vec<Self>) -> ::std::collections::HashMap<&'static str, houdini_node::RawAttribute> {
                let #multiunzip_pattern: (#multiunzip_types) =
                    houdini_node::itertools::multiunzip(entities.into_iter().map(#entity_map));

                std::collections::HashMap::from([
                    #(#hashmap_entries),*
                ])
            }
        }
    };
    generated.into()
}

fn get_field_name(field: &syn::Field) -> String {
    // Check for #[attr(name = "custom_name")] attribute
    for attr in &field.attrs {
        if attr.path().is_ident("attr") {
            if let Meta::List(meta_list) = &attr.meta {
                // Parse name = "value" format
                if let Ok(Meta::NameValue(name_value)) = syn::parse2(meta_list.tokens.clone()) {
                    if name_value.path.is_ident("name") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = name_value.value
                        {
                            return lit_str.value();
                        }
                    }
                }
            }
        }
    }
    // Fall back to field name
    field.ident.as_ref().unwrap().to_string()
}
