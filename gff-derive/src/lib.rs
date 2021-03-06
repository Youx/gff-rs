use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro;
#[macro_use]
extern crate quote;

/* {{{ GFF Structs */

struct GFFStructId(syn::LitInt);

impl syn::parse::Parse for GFFStructId {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let st_id = content.parse()?;
        Ok(GFFStructId(st_id))
    }
}

#[proc_macro_derive(GFFStruct, attributes(GFFStructId))]
pub fn derive_gff_struct(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // parse attribute GFFStructId
    let attribute = input.attrs.iter().find(
        |a| a.path.segments.len() == 1 && a.path.segments[0].ident == "GFFStructId"
    ).expect("GFFStructId attribute required for deriving GFFStruct");
    let parameters: GFFStructId = syn::parse2(attribute.tokens.clone())
        .expect("Invalid GFFStructId attribute!");

    let struct_name = &input.ident;
    let input = input.data;
    let struct_id = parameters.0;

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
                        /* deserializing from GffStruct to custom structure. */
                        impl std::convert::TryFrom<&GffFieldValue> for #struct_name {
                            type Error = &'static str;

                            fn try_from(value: &GffFieldValue) -> Result<Self, self::Error> {
                                match value {
                                    ::gff::common::GffFieldValue::Struct(s) => 
                                        ::gff::common::Deserialize::deserialize(s),
                                    _ => Err("Expected Struct"),
                                }
                            }
                        }
                        impl ::gff::common::Deserialize for #struct_name {
                            fn deserialize(s: &::gff::common::GffStruct)
                                -> Result<Self, &'static str> where Self: std::marker::Sized {
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

                        /* serializing from custom structure to GffStruct. */
                        impl ::gff::common::Serialize for #struct_name {
                            fn serialize(&self) -> Result<GffStruct, &'static str> {
                                Ok(GffStruct {
                                    st_type: #struct_id,
                                    fields: HashMap::from([
                                        #(
                                            (#keys.to_string(), (&self.#fields).try_into()?)
                                        ),*
                                    ])
                                })
                            }
                        }
                        impl std::convert::TryInto<GffFieldValue> for &#struct_name {
                            type Error = &'static str;

                            fn try_into(self) -> Result<GffFieldValue, self::Error> {
                                Ok(GffFieldValue::Struct(::gff::common::Serialize::serialize(self)?))
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

#[proc_macro_derive(GFFStructPack, attributes(GFFStructId))]
pub fn derive_gff_struct_pack(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // parse attribute GFFStructId
    let attribute = input.attrs.iter().find(
        |a| a.path.segments.len() == 1 && a.path.segments[0].ident == "GFFStructId"
    ).expect("GFFStructId attribute required for deriving GFFStruct");
    let parameters: GFFStructId = syn::parse2(attribute.tokens.clone())
        .expect("Invalid GFFStructId attribute!");

    let struct_name = &input.ident;
    let input = input.data;
    let struct_id = parameters.0;

    match input {
        syn::Data::Enum(_) => { panic!("Expected struct, got enum"); }
        syn::Data::Union(_) => { panic!("Expected struct, got union"); }
        syn::Data::Struct(data_struct) => {
            let field_count = data_struct.fields.len();
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

                    let pack_field_idx = match field_count {
                        0..=1 => quote! {
                            packer.data.structs.extend_from_slice(
                                &(packer.data.header.fields.1 as u32).to_le_bytes()
                            );
                        },
                        _ => quote! {
                            packer.data.structs.extend_from_slice(
                                &(packer.data.header.field_indices.1 as u32).to_le_bytes()
                            );
                        },
                    };

                    // Build the output, possibly using quasi-quotation
                    let expanded = quote! {
                        impl <'a, W: std::io::Write> gff::packer::PackField<'a, W> for #struct_name {
                            fn pack_field(&'a self, label: String, packer: &mut gff::packer::Packer<W>,
                                structs: &mut Vec<&'a dyn gff::packer::PackStruct<W>>, st_idx: &mut u32)
                                -> ()
                            {
                                let label_idx = packer.pack_label(&label).unwrap();

                                *st_idx += 1;
                                packer.pack_val_4(14, label_idx, &(*st_idx).to_le_bytes());
                                structs.push(self);
                            }
                        }

                        /* serializing from custom structure to GffStruct. */
                        impl<'a, W: std::io::Write> ::gff::packer::PackStruct<'a, W> for #struct_name {
                            fn pack(&'a self, packer: &mut ::gff::packer::Packer<W>,
                                structs: &mut Vec<&'a dyn ::gff::packer::PackStruct<W>>,
                                st_idx: &mut u32)
                                -> ()
                            {
                                // pack struct id
                                packer.data.structs.extend_from_slice(
                                    &(#struct_id as u32).to_le_bytes()
                                );
                                // pack field indices / field id
                                #pack_field_idx
                                // pack field count
                                packer.data.structs.extend_from_slice(
                                    &(#field_count as u32).to_le_bytes()
                                );
                                // pack fields
                                #(
                                    gff::packer::PackField::pack_field(
                                        &self.#fields, #keys.to_string(),
                                        packer, structs, st_idx
                                    );
                                )*;
                                // Ok()
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

/* }}} */
