use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

#[proc_macro_attribute]
pub fn inherit(parent_struct_tokens: TokenStream, child_struct: TokenStream) -> TokenStream {
    let parent_struct_ast = syn::parse::<DeriveInput>(parent_struct_tokens).unwrap();
 
    // Check the coming child_struct is a struct only
    if let Ok(child_ast) = syn::parse::<syn::DeriveInput>(child_struct) {
        // Check the coming parent_struct is a struct only
        return make_inheritance(&parent_struct_ast, &child_ast);
    }
    panic!("macro can be applied only to struct");
}

fn make_inheritance(parent_ast: &syn::DeriveInput, child_ast: &syn::DeriveInput) -> TokenStream {
    let child_struct_name = &child_ast.ident;

    match &child_ast.data {
        syn::Data::Struct(child) => {
            match &parent_ast.data {
                syn::Data::Struct(parent) => {
                    let mut new_struct_fields: Vec<syn::Field> = Vec::new();
                    // Loop through parent fields
                    for field in parent.fields.iter() {
                        new_struct_fields.push(field.clone());
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
                    panic!("Not implemented");
                }
            }
        }
        _ => {
            panic!("Method not implemented");
        }
    }
}
