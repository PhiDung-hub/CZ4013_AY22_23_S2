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

pub fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(serde::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let dummy_indent = Ident::new(
        &format!("_IMPL_DESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );
    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_indent: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: serde::__private::Option<#ident #ty_generics>,
            }

            impl #impl_generics serde::Deserialize for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut serde::__private::Option<Self>) -> &mut dyn serde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut serde::__private::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics serde::de::Visitor for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> serde::Result<serde::__private::Box<dyn serde::de::Map + '_>> {
                    Ok(serde::__private::Box::new(__State {
                        #(
                            #fieldname: serde::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: serde::__private::Option<#fieldty>,
                )*
                __out: &'__a mut serde::__private::Option<#ident #ty_generics>,
            }

            impl #wrapper_impl_generics serde::de::Map for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &serde::__private::str) -> serde::Result<&mut dyn serde::de::Visitor> {
                    match __k {
                        #(
                            #fieldstr => serde::__private::Ok(serde::Deserialize::begin(&mut self.#fieldname)),
                        )*
                        _ => serde::__private::Ok(<dyn serde::de::Visitor>::ignore()),
                    }
                }

                fn finish(&mut self) -> serde::Result<()> {
                    #(
                        let #fieldname = self.#fieldname.take().ok_or(serde::Error)?;
                    )*
                    *self.__out = serde::__private::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    serde::__private::Ok(())
                }
            }
        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
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

    let dummy_indent = Ident::new(
        &format!("_IMPL_DESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_indent: () = {
            #[repr(C)]
            struct __Visitor {
                __out: serde::__private::Option<#ident>,
            }

            impl serde::Deserialize for #ident {
                fn begin(__out: &mut serde::__private::Option<Self>) -> &mut dyn serde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut serde::__private::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl serde::de::Visitor for __Visitor {
                fn string(&mut self, s: &serde::__private::str) -> serde::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => return serde::__private::Err(serde::Error),
                    };
                    self.__out = serde::__private::Some(value);
                    serde::__private::Ok(())
                }
            }
        };
    })
}
