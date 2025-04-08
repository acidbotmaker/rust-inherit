use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::{collections::HashMap, fs};
use syn::{
    self, parse_file, parse_str, Data, DataStruct, DeriveInput, File, ImplItem, Item, ItemStruct,
};


// Declaring mods
mod struct_utils;

// Local imports
use struct_utils::StructHashMapItem;

// Constants
const MAIN_FILE: &str = "src/main.rs";
type OptionalChildImpl = Option<TokenStream>;

fn get_impl_s_item_name(item: &ImplItem) -> String {
    match item {
        ImplItem::Fn(method) => method.sig.ident.to_string(),
        ImplItem::Const(const_item) => const_item.ident.to_string(),
        ImplItem::Type(type_item) => type_item.ident.to_string(),
        _ => panic!("Not implemented"),
    }
}


fn get_item_index_from_array(item: ImplItem, impls_to_implement: & Vec<ImplItem>) -> Option<usize> {
    for (index, impl_item) in impls_to_implement.iter().enumerate() {
        if get_impl_s_item_name(impl_item) == get_impl_s_item_name(&item) {
            return Some(index);
        }
    }
    return None;
}

fn load_impl_from_parents(parent_impls: &Vec<ImplItem>, impls_to_implement: &mut Vec<ImplItem>) {
    for impl_item in parent_impls {
        // Get item name
        let item_name = get_impl_s_item_name(impl_item);

        // Check if impl item already exists
        let impl_item_index = get_item_index_from_array(impl_item.to_owned(), impls_to_implement);

        if let Some(item_index) = impl_item_index {
            impls_to_implement[item_index] = impl_item.clone();
        }else{
            &impls_to_implement.push(impl_item.clone());
        }
    }
}

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
) -> (DeriveInput, OptionalChildImpl) {
    let child_struct_name = &child_ast.ident;

    let mut impls_to_implement: Vec<ImplItem> = Vec::new();

    let mut parent_structs = Vec::new();
    for parent_struct_name in parent_struct_names {
        if let Some(parent_struct) = global_struct_hashmap.get(parent_struct_name) {
            let mut mod_struct_w_impl: Option<TokenStream> = None;

            // Check if parent has parent, i.e., if child has grandparent
            if parent_struct.parents.len() > 0 {
                println!("\t{}-{}---GRANDPARENT CHECK--------------------------------------------------------------", child_struct_name, parent_struct_name);
                let (mod_struct, _child_impl) = make_inheritance(
                    &parent_struct.parents,
                    &parent_struct.code,
                    global_struct_hashmap,
                );
                // mod_struct_w_impl = _child_impl;
                // // println!("Rectangle class: {:#?}", &mod_struct.clone().into_token_stream().to_string());
                println!("\t{}-{}---GRANDPARENT FINISHED--------------------------------------------------------------", child_struct_name, parent_struct_name);
                parent_structs.push(mod_struct);
            }

            // Load all impls of parent
            for impl_item in &parent_struct.impl_items {
                // Get item name
                let item_name = get_impl_s_item_name(impl_item);
                

                // TODO: Convert it into generic function
                // Check if impl item already exists
                let mut impl_item_index = 0;
                let mut impl_item_exists = false;

                for exiting_impl_item in &impls_to_implement {
                    let existing_impl_item_name = get_impl_s_item_name(exiting_impl_item);

                    if existing_impl_item_name == item_name {
                        impl_item_exists = true;
                        break;
                    }
                    impl_item_index += 1;
                }

                // Then add it
                if impl_item_exists {
                    impls_to_implement[impl_item_index] = impl_item.clone();
                } else {
                    impls_to_implement.push(impl_item.clone());
                }
            }

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
                    _ => panic!("Not implemented inheritance"),
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

            // Field loading completed, now do impl
            // Load child impls
            let child_struct = global_struct_hashmap
                .get(&child_struct_name.to_string())
                .unwrap();
            for impl_item in &child_struct.impl_items {
                // Get item name
                let item_name = get_impl_s_item_name(impl_item);

                // Check if impl item already exists
                let mut impl_item_index = 0;
                let mut impl_item_exists = false;

                for exiting_impl_item in &impls_to_implement {
                    let existimg_impl_item_name = get_impl_s_item_name(exiting_impl_item);

                    if existimg_impl_item_name == item_name {
                        impl_item_exists = true;
                        break;
                    }
                    impl_item_index += 1;
                }

                // Then add it
                if impl_item_exists {
                    impls_to_implement[impl_item_index] = impl_item.clone();
                } else {
                    impls_to_implement.push(impl_item.clone());
                }
            }

            let gen = (quote! {
                #[derive(Debug)]
                struct #child_struct_name {
                    #(#new_struct_fields),*
                }
            })
            .into_token_stream()
            .to_string();

            if let Ok(ast) = parse_str(&gen) {
                let mut all_func_sigs = Vec::new();

                for impl_item in &impls_to_implement {
                    if let ImplItem::Fn(func) = impl_item {
                        let func_signature = &func.sig;
                        // Add semicolon at the end
                        let function_signature_string =
                            func_signature.to_token_stream().to_string() + ";";
                        let parsed_function_signature =
                            parse_str::<ImplItem>(&function_signature_string).unwrap();
                        all_func_sigs.push(parsed_function_signature);
                    }
                }

                let custom_trait_name =
                    format_ident!("CustomTraitFromForChild{}", child_struct_name);
                let custom_trait_impl = (quote! {
                    #[derive(Debug)]
                    struct #child_struct_name {
                        #(#new_struct_fields),*
                    }

                    trait #custom_trait_name {
                        // Add semicolon at the end
                        #(#all_func_sigs)*
                    }

                    impl #custom_trait_name for #child_struct_name {
                        #(#impls_to_implement)*
                    }
                })
                .into();
                return (ast, Some(custom_trait_impl));
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
        println!("{}---STARTED--------------------------------------------------------------", child_ast.ident.to_string());
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
        let (inherited_child_struct, child_impl) =
            make_inheritance(&parent_struct_names, &child_ast, &global_struct_hashmap);
        
        // Convert deriveInput to TokenStream
        let inherited_child_struct_tokenstream = inherited_child_struct.into_token_stream().into();
        
        
        println!("{}---FINISHED--------------------------------------------------------------", child_ast.ident.to_string());
        if let Some(child_impl) = child_impl {
            return child_impl;
        }
        return inherited_child_struct_tokenstream;
    }
    panic!("`inherit` macro can be applied only to struct");
}
