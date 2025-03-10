use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::HashMap, fs};
use syn::{self, parse_file, parse_macro_input, parse_str, Data, DataStruct, DeriveInput, File, Item, ItemStruct};
use lazy_static::lazy_static;
use std::sync::Mutex;

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

struct StructHashMapItem {
    code: DeriveInput,
    parents: Vec<String>,
}

// NOTE: Cache it maybe?
fn load_all_struct_hashmap() -> HashMap<String, StructHashMapItem> {
    let code = fs::read_to_string(MAIN_FILE).expect("Failed to read file");

    // Parse to ast
    let ast: File = parse_file(&code).expect("Failed to parse file");

    let mut struct_hashmap: HashMap<String, StructHashMapItem> = HashMap::new();
    for item in ast.items {
        if let Item::Struct(parsed_struct) = item {
            let struct_name = parsed_struct.ident.to_string();
            let mut item = StructHashMapItem{
                code: item_struct_to_derive_input(parsed_struct.clone()),
                parents: Vec::new(),
            };

            // Check for parent here
            let attrs = parsed_struct.attrs.clone();
            for attr in attrs {
                let meta = attr.meta;
                if meta.path().is_ident("inherit"){
                    let attribute = meta.to_token_stream().to_string().replace("inherit(", "").replace(")", "");
                    // println!("{} -> {}", parsed_struct.ident.to_string(), attribute);
                    // Split by , and trim attribute
                    let _ = attribute.split(',').map(|s| item.parents.push(s.trim().to_string())).collect::<Vec<_>>();
                    break;
                }
            }


            // Check if struct is already present in hashmap or not
            if !struct_hashmap.contains_key(&struct_name) {
                struct_hashmap.insert(struct_name, item);
            }
        }
    }

    return struct_hashmap;
}

fn make_parent_inhert(parent_struct_names: &Vec<String>, child_ast: &DeriveInput) -> DeriveInput {
    let child_struct_name = &child_ast.ident;

    // Load parent structs from hashmap
    let global_struct_hashmap = load_all_struct_hashmap();
    let mut parent_structs = Vec::new();
    for parent_struct_name in parent_struct_names {
        if let Some(parent_struct) = global_struct_hashmap.get(parent_struct_name) {

            // Check if parent has grandparent
            if parent_struct.parents.len() > 0 {
                let ss = make_parent_inhert(&parent_struct.parents, &parent_struct.code);
                parent_structs.push(ss);
            }

            parent_structs.push(parent_struct.code.clone());
        }
    }

    match &child_ast.data {
        syn::Data::Struct(child) => {
            
            let mut new_struct_fields: Vec<syn::Field> = Vec::new();
            
            for parent_ast in parent_structs{
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

            let gen = (quote! {
                #[derive(Debug)]
                struct #child_struct_name {
                    #(#new_struct_fields),*
                }
            }).into_token_stream().to_string();

            if let Ok(ast) = parse_str(&gen) {
                return ast;
            }
            
            panic!("Failed to parse generated code");
        }
        _ => {
            panic!("Method not implemented");
        }
    }
}


#[proc_macro_attribute]
pub fn inherit(parent_struct_tokens: TokenStream, child_struct: TokenStream) -> TokenStream {
    let global_struct_hashmap = load_all_struct_hashmap();

    let parent_struct_names = parent_struct_tokens.to_string().split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();
    
    let mut parent_structs = Vec::new();

    // Load parent structs
    for parent_struct_name in parent_struct_names {
        if let Some(parent_struct) = global_struct_hashmap.get(&parent_struct_name) {
            let ss = make_parent_inhert(&parent_struct.parents, &parent_struct.code);
            parent_structs.push(ss);
        }
    }

    // load_global_ast(parent_struct_tokens.to_string());
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
