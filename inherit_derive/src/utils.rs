
use quote::ToTokens;
use syn::{
    self,ImplItem
};

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

fn check_and_replace_item_in_existing_array<T>(mut impls: Vec<T>, new_item: &T, get_item_name: fn(&T) -> String) -> Vec<T>
where T: Clone
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

pub fn merge_old_array_in_new_array<T>(mut new_array: Vec<T>, old_array: &Vec<T>, get_item_name: fn(&T) -> String) -> Vec<T>
where T: Clone
{
    for item in old_array {
        new_array = check_and_replace_item_in_existing_array(new_array, &item, get_item_name);
    }
    return new_array;
}
