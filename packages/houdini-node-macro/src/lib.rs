use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(EntityFromAttribute)]
pub fn derive_entity_from_attribute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_entity_from_attribute(&input)
}

#[proc_macro_derive(EntityIntoAttribute)]
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
            let field_name_str = field_name.to_string();
            quote! {
                let #field_name = load_from_attr(attrs.remove(#field_name_str).unwrap())?;
            }
        })
        .collect();

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_construction = quote! {
        izip!(#(#field_names),*).map(|(#(#field_names),*)| Self { #(#field_names),* })
    };

    let generated = quote! {
        impl EntityFromAttribute for #name {
            fn from_attr(
                mut attrs: std::collections::HashMap<String, RawAttribute>,
            ) -> Result<impl Iterator<Item = Self>> {
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
    let field_name_strs: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let vec_types = fields.iter().map(|_| quote! { Vec<_>});

    let multiunzip_pattern = quote! { (#(#field_names,)*) };
    let multiunzip_types = quote! { #(#vec_types,)* };

    let entity_map = quote! { |entity| (#(entity.#field_names,)*) };

    let hashmap_entries: Vec<_> = field_names
        .iter()
        .zip(field_name_strs.iter())
        .map(|(name, name_str)| {
            quote! { (#name_str, generate_to_attr(#name)) }
        })
        .collect();

    let generated = quote! {
        impl EntityIntoAttribute for #name {
            fn into_attr(entities: Vec<Self>) -> ::std::collections::HashMap<&'static str, houdini_node::RawAttribute> {
                let #multiunzip_pattern: (#multiunzip_types) =
                    multiunzip(entities.into_iter().map(#entity_map));

                std::collections::HashMap::from([
                    #(#hashmap_entries),*
                ])
            }
        }
    };
    generated.into()
}
