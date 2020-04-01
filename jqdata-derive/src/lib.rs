//! Defines derive macro to generate implementions of 
//! each request type defines in jqdata-model crate.

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2;
use quote::*;
use syn::{parse_macro_input, DeriveInput};

/// entrypoint of derive macro to implements HasMethod and BodyConsumer traits on
/// marked structs
#[proc_macro_derive(Jqdata, attributes(method, consume))]
pub fn derive_jqdata(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let result = match ast.data {
        syn::Data::Struct(ref s) => derive_jqdata_for_struct(&ast, &s.fields),
        _ => panic!("doesn't work with enums or unions yet"),
    };
    TokenStream::from(result)
}

fn derive_jqdata_for_struct(
    ast: &syn::DeriveInput,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    match *fields {
        syn::Fields::Named(..) => impl_jqdata_for_struct(&ast),
        syn::Fields::Unit => impl_jqdata_for_struct(&ast),
        syn::Fields::Unnamed(..) => panic!("doesn't work with unnamed fields yet"),
    }
}

fn impl_jqdata_for_struct(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &ast.ident;

    let request_method = ast
        .attrs
        .iter()
        .find_map(|attr| {
            if let Ok(syn::Meta::List(metalist)) = attr.parse_meta() {
                if let Some(ident) = metalist.path.get_ident() {
                    if ident == "method" {
                        if metalist.nested.len() != 1 {
                            panic!("must have one method name in request attribute");
                        }
                        return metalist.nested.first().map(nested_meta_to_string);
                    }
                }
            }
            None
        })
        .expect("must have request attribute with method name");
    
    let consume_meta = ast
        .attrs
        .iter()
        .find_map(|attr| {
            if let Ok(syn::Meta::List(metalist)) = attr.parse_meta() {
                if let Some(ident) = metalist.path.get_ident() {
                    if ident == "consume" {
                        return Some(metalist.nested);
                    }
                }
            }
            None
        })
        .expect("must have response attribute with method name");
    let consume_format = consume_meta
        .iter()
        .find_map(|m| {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = m {
                if nv.path.is_ident("format") {
                    if let syn::Lit::Str(ref strlit) = nv.lit {
                        return Some(strlit.value());
                    }
                }
            }
            None
        })
        .expect("format must be set in response attribute");

    let ty = consume_meta.iter().find_map(|m| {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = m {
            if nv.path.is_ident("type") {
                if let syn::Lit::Str(ref strlit) = nv.lit {
                    return Some(strlit.value());
                }
            }
        }
        None
    });

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let (consume_impl, output_ty) = match consume_format.as_ref() {
        "csv" => {
            let ty = ty.expect("type must be set in response attribute when format is csv");
            let single_ty: syn::Type = syn::parse_str(&ty.to_string())
                .expect("invalid type in response attribute");
            let output_ty: syn::Type = syn::parse_str(&format!("Vec<{}>", ty))
                .expect("invalid type in response attribute");
            let consume_impl = quote! {
                impl #impl_generics crate::models::CsvListBodyConsumer for #struct_name #ty_generics #where_clause {
                    type Output = #single_ty;
                } 
            };
            (consume_impl, output_ty)
        }
        "line" => {
            if ty.is_some() {
                panic!("type should not be set in response attribute when format is line");
            }
            let output_ty: syn::Type = syn::parse_str("Vec<String>").unwrap();
            let consume_impl = quote! {
                impl #impl_generics crate::models::LineBodyConsumer for #struct_name #ty_generics #where_clause {}
            };
            (consume_impl, output_ty)
        }
        "single" => {
            let output_ty = ty.expect("type must be set in response attribute when format is single");
            let output_ty: syn::Type = syn::parse_str(&output_ty).expect("invalid type in response attribute");
            let consume_impl = quote! {
                impl #impl_generics crate::models::SingleBodyConsumer<#output_ty> for #struct_name #ty_generics #where_clause {}
            };
            (consume_impl, output_ty)
        }
        "json" => {
            let output_ty = ty.expect("type must be set in response attribute when format is json");
            let output_ty: syn::Type = syn::parse_str(&output_ty).expect("invalid type in response attribute");
            let consume_impl = quote! {
                impl #impl_generics crate::models::JsonBodyConsumer for #struct_name #ty_generics #where_clause {
                    type Output = #output_ty;
                }
            };
            (consume_impl, output_ty)
        },
        _ => panic!("format {} not supported", consume_format),
    };

    quote! {
        impl #impl_generics crate::models::HasMethod for #struct_name #ty_generics #where_clause {
            fn method(&self) -> String {
                #request_method.to_owned()
            }
        }

        impl #impl_generics crate::models::BodyConsumer<#output_ty> for #struct_name #ty_generics #where_clause {
            fn consume_body<R: std::io::Read>(body: R) -> crate::Result<#output_ty> {
                Self::consume(body)
            }
        }

        #consume_impl
    }
}

fn nested_meta_to_string(nm: &syn::NestedMeta) -> String {
    match nm {
        syn::NestedMeta::Meta(meta) => match meta {
            syn::Meta::Path(path) => path.get_ident().as_ref().unwrap().to_string(),
            _ => panic!("must be single path"),
        },
        syn::NestedMeta::Lit(lit) => match lit {
            syn::Lit::Str(litstr) => {
                litstr.value()
            }
            _ => panic!("must be string literal"),
        },
    }
}
