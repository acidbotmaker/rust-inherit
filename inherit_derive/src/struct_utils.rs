use quote::ToTokens;
use syn::{
    self, Data, DataStruct, DeriveInput, ItemStruct, ImplItem
};


fn item_struct_to_derive_input(item_struct: ItemStruct) -> DeriveInput {
    DeriveInput {
        attrs: item_struct.attrs,       // Preserve attributes
        vis: item_struct.vis,           // Preserve visibility (pub/private)
        ident: item_struct.ident,       // Struct name
        generics: item_struct.generics, // Generic parameters
        data: Data::Struct(DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields, // Fields of the struct
            semi_token: item_struct.semi_token,
        }),
    }
}

pub struct StructHashMapItem {
    pub code: DeriveInput,
    pub parents: Vec<String>,
    pub impl_items: Vec<ImplItem>,
}

impl StructHashMapItem {
    pub fn new(item_struct: &ItemStruct) -> Self {
        let mut new_struct = Self {
            code: item_struct_to_derive_input(item_struct.clone()),
            parents: Vec::new(),
            impl_items: Vec::new(),
        };

        // Check for parent here
        let attrs = item_struct.attrs.clone();
        for attr in attrs {
            let meta = attr.meta;
            if meta.path().is_ident("inherit") {
                let attribute = meta
                    .to_token_stream()
                    .to_string()
                    .replace("inherit(", "")
                    .replace(")", "");
                // println!("{} -> {}", parsed_struct.ident.to_string(), attribute);
                // Split by , and trim attribute
                let _ = attribute
                    .split(',')
                    .map(|s| new_struct.parents.push(s.trim().to_string()))
                    .collect::<Vec<_>>();
                break;
            }
        }
        return new_struct;
    }
}
