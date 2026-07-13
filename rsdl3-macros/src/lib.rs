use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Error, FnArg, GenericArgument, ItemFn, PathArguments, ReturnType, Type,
};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);

    if let Err(err) = validate_function(&function) {
        return err.to_compile_error().into();
    }

    let original_name = function.sig.ident.clone();
    let inner_name = format_ident!("__rsdl3_main_{}", original_name);
    function.sig.ident = inner_name.clone();

    let call = match function.sig.inputs.len() {
        0 => quote! { #inner_name() },
        1 => quote! {
            #inner_name(::rsdl3::runtime::Args::from_raw(argc, argv))
        },
        _ => unreachable!(),
    };

    quote! {
        #function

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn SDL_main(
            argc: ::core::ffi::c_int,
            argv: *mut *mut ::core::ffi::c_char,
        ) -> ::core::ffi::c_int {
            match #call {
                ::core::result::Result::Ok(()) => 0,
                ::core::result::Result::Err(_) => 1,
            }
        }
    }
    .into()
}

fn validate_function(function: &ItemFn) -> Result<(), Error> {
    if function.sig.constness.is_some() {
        return Err(Error::new_spanned(
            &function.sig.constness,
            "rsdl3::main does not support const functions",
        ));
    }
    if function.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            &function.sig.asyncness,
            "rsdl3::main does not support async functions",
        ));
    }
    if function.sig.unsafety.is_some() {
        return Err(Error::new_spanned(
            &function.sig.unsafety,
            "rsdl3::main does not support unsafe functions",
        ));
    }
    if !function.sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &function.sig.generics,
            "rsdl3::main does not support generic functions",
        ));
    }
    if function.sig.variadic.is_some() {
        return Err(Error::new_spanned(
            &function.sig.variadic,
            "rsdl3::main does not support variadic functions",
        ));
    }
    if function.sig.inputs.len() > 1 {
        return Err(Error::new_spanned(
            &function.sig.inputs,
            "rsdl3::main expects zero arguments or one Args argument",
        ));
    }
    for input in &function.sig.inputs {
        if matches!(input, FnArg::Receiver(_)) {
            return Err(Error::new_spanned(
                input,
                "rsdl3::main cannot be used on methods",
            ));
        }
    }
    match &function.sig.output {
        ReturnType::Type(_, ty) if is_result_unit(ty) => Ok(()),
        ReturnType::Type(_, ty) => Err(Error::new_spanned(
            ty,
            "rsdl3::main function must return Result<(), E>",
        )),
        ReturnType::Default => Err(Error::new_spanned(
            &function.sig,
            "rsdl3::main function must return Result<(), E>",
        )),
    }
}

fn is_result_unit(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() {
        return false;
    }

    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Result" {
        return false;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    let mut args = args.args.iter();

    matches!(args.next(), Some(GenericArgument::Type(Type::Tuple(tuple))) if tuple.elems.is_empty())
        && args.next().is_some()
        && args.next().is_none()
}
