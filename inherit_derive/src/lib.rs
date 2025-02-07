use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use syn::{self, parse_file, Data, DataStruct, DeriveInput, File, Item, ItemStruct};

const MAIN_FILE: &str = "src/main.rs";

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

fn load_global_ast(parent_struct_names_string: String) -> Vec<DeriveInput> {
    let parent_struct_names: Vec<String> = parent_struct_names_string
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let code = fs::read_to_string(MAIN_FILE).expect("Failed to read file");

    // Parse to ast
    let ast: File = parse_file(&code).expect("Failed to parse file");

    let mut parent_structs: Vec<DeriveInput> = Vec::new();
    for item in ast.items {
        if let Item::Struct(parsed_struct) = item {
            if parent_struct_names.contains(&parsed_struct.ident.to_string()) {
                parent_structs.push(item_struct_to_derive_input(parsed_struct.clone()));
            }
        }
    }

    return parent_structs;
}

#[proc_macro_attribute]
pub fn inherit(parent_struct_tokens: TokenStream, child_struct: TokenStream) -> TokenStream {
    let parent_structs = load_global_ast(parent_struct_tokens.to_string());

    if parent_structs.len() == 0 {
        panic!(
            "Parent(s) {} not found in {}",
            parent_struct_tokens.to_string(),
            MAIN_FILE
        );
    }

    // Check the coming child_struct is a struct only
    if let Ok(child_ast) = syn::parse::<syn::DeriveInput>(child_struct) {
        // Check the coming parent_struct is a struct only
        return make_inheritance(&parent_structs, &child_ast);
    }
    panic!("macro can be applied only to struct");
}

fn make_inheritance(parent_asts: &Vec<syn::DeriveInput>, child_ast: &syn::DeriveInput) -> TokenStream {
    let child_struct_name = &child_ast.ident;

    match &child_ast.data {
        syn::Data::Struct(child) => {
            
            let mut new_struct_fields: Vec<syn::Field> = Vec::new();
            
            for parent_ast in parent_asts{
                match &parent_ast.data {
                    syn::Data::Struct(parent) => {
                        // Loop through parent fields
                        for field in parent.fields.iter() {
                            // Check if field already exists in new_struct_fields
                            let mut field_index = 0;
                            let mut change_field = false;
                            for new_field in new_struct_fields.iter() {
                                if field.ident == new_field.ident {
                                    change_field = true;
                                    break;
                                }
                                field_index += 1;
                            }
    
                            // Remove it
                            if change_field {
                                new_struct_fields.remove(field_index);
                            }
    
                            // Then add parent's
                            new_struct_fields.push(field.clone());
                        }
                    }
                    _ => panic!("Not implemented inheritance")
                }
            }
           
            // Loop through child fields
            for field in child.fields.iter() {
                // Check if field already exists in new_struct_fields
                let mut field_index = 0;
                let mut change_field = false;
                for new_field in new_struct_fields.iter() {
                    if field.ident == new_field.ident {
                        change_field = true;
                        break;
                    }
                    field_index += 1;
                }

                // Remove it
                if change_field {
                    new_struct_fields.remove(field_index);
                }

                // Then add parent's
                new_struct_fields.push(field.clone());
            }

            let gen = quote! {
                #[derive(Debug)]
                struct #child_struct_name {
                    #(#new_struct_fields),*
                }
            };
            return gen.into();
        }
        _ => {
            panic!("Method not implemented");
        }
    }
}
