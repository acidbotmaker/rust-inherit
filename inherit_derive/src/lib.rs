use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::HashMap, fs};
use syn::{self, parse_file, parse_str, DeriveInput, File, ImplItem, Item};

// Declaring mods
mod struct_;
mod utils;

// Local imports
use struct_::StructHashMapItem;
use utils::{
    get_impl_s_item_name, get_struct_field_name, make_trait_and_impl_from_impls,
    merge_old_array_in_new_array,
};

// Constants
const MAIN_FILE: &str = "src/main.rs";

// NOTE: Cache it maybe?
fn load_all_struct_hashmap() -> HashMap<String, StructHashMapItem> {
    let code = fs::read_to_string(MAIN_FILE).expect("Failed to read file");

    // Parse to ast
    let ast: File = parse_file(&code).expect("Failed to parse file");

    let mut struct_hashmap: HashMap<String, StructHashMapItem> = HashMap::new();
    for item in &ast.items {
        if let Item::Struct(parsed_struct) = item {
            let struct_name = parsed_struct.ident.to_string();
            let item = StructHashMapItem::new(parsed_struct);

            // Check if struct is already present in hashmap or not
            if !struct_hashmap.contains_key(&struct_name) {
                struct_hashmap.insert(struct_name, item);
            }
        }
    }

    // Load all impls
    for item in &ast.items {
        if let Item::Impl(parsed_impl) = item {
            if let syn::Type::Path(type_path) = &*parsed_impl.self_ty {
                // Extract the last segment of the path, which is usually struct
                if let Some(segment) = type_path.path.segments.last() {
                    let struct_name = segment.ident.to_string();
                    if struct_hashmap.contains_key(&struct_name) {
                        let struct_to_mod = struct_hashmap.get_mut(&struct_name).unwrap();

                        for item in &parsed_impl.items {
                            struct_to_mod.impl_items.push(item.clone());
                        }
                    }
                }
            }
        }
    }

    return struct_hashmap;
}

fn make_inheritance(
    parent_struct_names: &Vec<String>,
    child_ast: &DeriveInput,
    global_struct_hashmap: &HashMap<String, StructHashMapItem>,
) -> (DeriveInput, Vec<ImplItem>) {
    let child_struct_name = &child_ast.ident;

    let mut impls_to_implement: Vec<ImplItem> = Vec::new();

    let mut parent_structs = Vec::new();
    for parent_struct_name in parent_struct_names {
        if let Some(parent_struct) = global_struct_hashmap.get(parent_struct_name) {
            // Check if parent has parent, i.e., if child has grandparent
            if parent_struct.parents.len() > 0 {
                println!("\t{} -> {}---GRANDPARENT CHECK--------------------------------------------------------------", child_struct_name, parent_struct_name);
                let (mod_struct, child_impls) = make_inheritance(
                    &parent_struct.parents,
                    &parent_struct.code,
                    global_struct_hashmap,
                );
                // Load parent's parent's impl in impls_to_implement array
                impls_to_implement = merge_old_array_in_new_array(
                    impls_to_implement,
                    &child_impls,
                    get_impl_s_item_name,
                );
                println!("\t{} -> {}---GRANDPARENT FINISHED--------------------------------------------------------------", child_struct_name, parent_struct_name);
                parent_structs.push(mod_struct);
            }

            // Load all impls of parent
            impls_to_implement = merge_old_array_in_new_array(
                impls_to_implement,
                &parent_struct.impl_items,
                get_impl_s_item_name,
            );

            parent_structs.push(parent_struct.code.clone());
            // TODO: Parse the incoming parent struct and pull methods from it as well
        }
    }

    match &child_ast.data {
        syn::Data::Struct(child) => {
            let mut new_struct_fields: Vec<syn::Field> = Vec::new();

            // Load all fields (parent + child)
            for parent_ast in parent_structs {
                match &parent_ast.data {
                    syn::Data::Struct(parent) => {
                        new_struct_fields = merge_old_array_in_new_array(
                            new_struct_fields,
                            &parent.fields.iter().map(|x| x.clone()).collect(),
                            get_struct_field_name,
                        );
                    }
                    _ => panic!("Not implemented inheritance"),
                }
            }

            // Loop through child fields
            new_struct_fields = merge_old_array_in_new_array(
                new_struct_fields,
                &child.fields.iter().map(|x| x.clone()).collect(),
                get_struct_field_name,
            );

            // Field loading completed, now do impl
            // Load child impls
            let child_struct = global_struct_hashmap
                .get(&child_struct_name.to_string())
                .unwrap();
            impls_to_implement = merge_old_array_in_new_array(
                impls_to_implement,
                &child_struct.impl_items,
                get_impl_s_item_name,
            );

            // Code generation starts here
            let gen = (quote! {
                #[derive(Debug)]
                struct #child_struct_name {
                    #(#new_struct_fields),*
                }
            })
            .into_token_stream()
            .to_string();

            if let Ok(ast) = parse_str(&gen) {
                return (ast, impls_to_implement);
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
    // Check the coming child_struct is a struct only
    if let Ok(child_ast) = syn::parse::<syn::DeriveInput>(child_struct) {
        println!(
            "{}---STARTED--------------------------------------------------------------",
            child_ast.ident.to_string()
        );
        // Load global struct hashmap
        let global_struct_hashmap = load_all_struct_hashmap();

        // Check if parent struct names are given
        let parent_struct_names = parent_struct_tokens
            .to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();
        if parent_struct_names.len() == 0 {
            panic!("At least one parent struct must be specified");
        }

        // Check if passed parent struct definition is available or not
        for parent_struct_name in &parent_struct_names {
            if !global_struct_hashmap.contains_key(parent_struct_name) {
                panic!(
                    "Parent struct {} not found in {}",
                    parent_struct_name, MAIN_FILE
                );
            }
        }

        // Make inheritance here
        let (inherited_child_struct, child_impls) =
            make_inheritance(&parent_struct_names, &child_ast, &global_struct_hashmap);

        println!(
            "{}---FINISHED--------------------------------------------------------------",
            child_ast.ident.to_string()
        );
        if child_impls.len() > 0 {
            return make_trait_and_impl_from_impls(&inherited_child_struct, &child_impls);
        }

        let inherited_child_struct_tokenstream = inherited_child_struct.into_token_stream().into();
        return inherited_child_struct_tokenstream;
    }
    panic!("`inherit` macro can be applied only to struct");
}
