use crate::{attr, bound};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Ident, Result,
};

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(&input, fields),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
        _ => Err(Error::new(
            Span::call_site(),
            "only named fields structs and variants enums are supported"
        ))
    }
}

fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy_indent = Ident::new(
        &format!("_IMPL_SERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;
    let index = 0usize..;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(serde::Serialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_indent: () = {
            impl #impl_generics serde::Serialize for #ident #ty_generics #bounded_where_clause {
                fn begin(&self) -> serde::ser::Fragment {
                    serde::ser::Fragment::Map(serde::__private::Box::new(__Map {
                        data: self,
                        state: 0,
                    }))
                }
            }

            struct __Map #wrapper_impl_generics #where_clause {
                data: &'__a #ident #ty_generics,
                state: serde::__private::usize,
            }

            impl #wrapper_impl_generics serde::ser::Map for __Map #wrapper_ty_generics #bounded_where_clause {
                fn next(&mut self) -> serde::__private::Option<(serde::__private::Cow<serde::__private::str>, &dyn serde::Serialize)> {
                    let __state = self.state;
                    self.state = __state + 1;
                    match __state {
                        #(
                            #index => serde::__private::Some((
                                serde::__private::Cow::Borrowed(#fieldstr),
                                &self.data.#fieldname,
                            )),
                        )*
                        _ => serde::__private::None,
                    }
                }
            }
        };
    })
}

fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_SERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let var_idents = enumeration
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "Invalid variant: only simple enum variants without fields are supported",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let names = enumeration
        .variants
        .iter()
        .map(attr::name_of_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            impl serde::Serialize for #ident {
                fn begin(&self) -> serde::ser::Fragment {
                    match self {
                        #(
                            #ident::#var_idents => {
                                serde::ser::Fragment::Str(serde::__private::Cow::Borrowed(#names))
                            }
                        )*
                    }
                }
            }
        };
    })
}
