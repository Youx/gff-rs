use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro;
#[macro_use]
extern crate quote;


#[proc_macro_derive(DeGFF)]
pub fn derive_gff_deserialize(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let input = input.data;

    match input {
        syn::Data::Enum(_) => { panic!("Expected struct, got enum"); }
        syn::Data::Union(_) => { panic!("Expected struct, got union"); }
        syn::Data::Struct(data_struct) => {
            match data_struct.fields {
                syn::Fields::Unnamed(_) => { panic!("Expected named fields, got unnamed"); }
                syn::Fields::Unit => { panic!("Expected named fields, got unit"); }
                syn::Fields::Named(named_fields) => {
                    let fields : Vec<&syn::Ident> = named_fields.named.iter().map(|field| {
                        field.ident.as_ref().unwrap()
                    }).collect();
                    let keys : Vec<String> = fields.iter().map(|ident| {
                        ident.to_string()
                    }).collect();

                    // Build the output, possibly using quasi-quotation
                    let expanded = quote! {
                        impl std::convert::TryFrom<&GffFieldValue> for #struct_name {
                            type Error = &'static str;

                            fn try_from(value: &GffFieldValue) -> Result<Self, self::Error> {
                                match value {
                                    ::gff::common::GffFieldValue::Struct(s) => ::gff::common::UnpackStruct::unpack(s),
                                    _ => Err("Expected Struct"),
                                }
                            }
                        }
                        impl ::gff::common::UnpackStruct for #struct_name {
                            fn unpack(s: &::gff::common::GffStruct) -> Result<Self, &'static str> where Self: std::marker::Sized {
                                Ok(#struct_name {
                                    #(
                                        #fields : std::convert::TryFrom::try_from(
                                            s.fields.get(#keys)
                                                    .ok_or("key not found")?
                                        )?
                                    ),*
                                })
                            }
                        }
                    };
                    // Hand the output tokens back to the compiler
                    TokenStream::from(expanded)
                }
            }
        }
    }


}

