use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{self, parse_str, DeriveInput, ImplItem};

// Some util functions to get field and impl names
pub fn get_impl_s_item_name(item: &ImplItem) -> String {
    match item {
        ImplItem::Fn(method) => method.sig.ident.to_string(),
        ImplItem::Const(const_item) => const_item.ident.to_string(),
        ImplItem::Type(type_item) => type_item.ident.to_string(),
        _ => panic!("Not implemented"),
    }
}

pub fn get_struct_field_name(field: &syn::Field) -> String {
    field.clone().ident.into_token_stream().to_string()
}

fn check_and_replace_item_in_existing_array<T>(
    mut impls: Vec<T>,
    new_item: &T,
    get_item_name: fn(&T) -> String,
) -> Vec<T>
where
    T: Clone,
{
    let item_name = get_item_name(new_item);
    let mut impl_item_index = 0;
    let mut impl_item_exists = false;

    for existing_item in &mut impls {
        let existing_item_name = get_item_name(&existing_item);
        if existing_item_name == item_name {
            impl_item_exists = true;
            break;
        }
        impl_item_index += 1;
    }

    if impl_item_exists {
        impls[impl_item_index] = new_item.clone();
    } else {
        impls.push(new_item.clone());
    }

    return impls;
}

pub fn merge_old_array_in_new_array<T>(
    mut new_array: Vec<T>,
    old_array: &Vec<T>,
    get_item_name: fn(&T) -> String,
) -> Vec<T>
where
    T: Clone,
{
    for item in old_array {
        new_array = check_and_replace_item_in_existing_array(new_array, &item, get_item_name);
    }
    return new_array;
}

pub fn make_trait_and_impl_from_impls(
    new_struct: &DeriveInput,
    impls_to_implement: &Vec<ImplItem>,
) -> TokenStream {
    // Load struct fields
    match &new_struct.data {
        // If the data type is struct
        syn::Data::Struct(child) => {
            let child_struct_name = &new_struct.ident;
            let new_struct_fields: Vec<syn::Field> =
                child.fields.iter().map(|x| x.clone()).collect();

            // Load function signature
            let mut all_func_sigs = Vec::new();
            for impl_item in impls_to_implement {
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

            let custom_trait_name = format_ident!("CustomTraitFromForChild{}", child_struct_name);
            return (quote! {
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
        }
        _ => panic!("Not implemented trait creation method for anything other than struct"),
    }
}
