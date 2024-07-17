extern crate proc_macro;
extern crate quote;
extern crate syn;

#[proc_macro_derive(DeriveErrors, attributes(errors, enum_fields))]
pub fn derive_errors(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = ast.ident;

    let mut errors: std::vec::Vec<syn::Path> = vec![];
    let mut enums: std::vec::Vec<syn::Ident> = vec![];

    for attr in ast.attrs {
        let syn::Attribute { meta, .. } = &attr;
        match meta {
            syn::Meta::List(list) if list.path.is_ident("errors") => {
                let punctuated_list_errors = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap();

                for (path, _) in punctuated_list_errors.into_pairs().map(|x| x.into_tuple()) {
                    errors.push(path);
                }
            }
            syn::Meta::List(list) if list.path.is_ident("enum_fields") => {
                let punctuated_list_errors = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated,
                    )
                    .unwrap();

                for (id, _) in punctuated_list_errors.into_pairs().map(|x| x.into_tuple()) {
                    enums.push(id);
                }
            }
            _ => {}
        }
    }

    let gen = quote::quote! {

        #(
            impl From<#errors> for #name{
                fn from(_value:#errors) -> Self{
                    #name::#enums
                }
            }
        )*
    };
    gen.into()
}
