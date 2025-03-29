use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

#[proc_macro_attribute]
pub fn first_char_variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    // Check if the input is an enum
    let Data::Enum(enum_data) = &mut input.data else {
        panic!("This macro can only be used with enums");
    };

    // Create new variants with first characters
    let mut new_variants = Vec::new();
    for variant in &enum_data.variants {
        let variant_name = variant.ident.to_string();
        let first_char = variant_name.chars().next().unwrap();
        let first_char_ident = Ident::new(&first_char.to_string(), variant.ident.span());
        
        let mut new_variant = variant.clone();
        new_variant.ident = first_char_ident;
        
        new_variants.push(new_variant);
    }

    // Extend the existing variants with new first-character variants
    enum_data.variants.extend(new_variants);
    
    // Reconstruct the enum with new variants
    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}