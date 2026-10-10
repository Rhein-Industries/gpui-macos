// Modified for gpui-macos: resolve the independently published fork packages
// and the public gpui-macos facade while retaining upstream package support.
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Resolve the consumer's GPUI API, preferring the Kit facade over a direct engine.
pub(crate) fn gpui() -> syn::Result<TokenStream> {
    for package in ["gpui-macos-kit", "gpui-kit"] {
        if let Ok(found) = crate_name(package) {
            return Ok(found_crate_path(package, found));
        }
    }
    if let Ok(found) = crate_name("gpui-macos") {
        let facade = found_crate_path("gpui-macos", found);
        return Ok(quote!(#facade::gpui_kit));
    }
    for package in ["gpui-pre", "gpui-fast"] {
        if let Ok(found) = crate_name(package) {
            return Ok(found_crate_path(package, found));
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "IntoPlot requires gpui-macos, gpui-macos-kit, gpui-kit, gpui-pre, or gpui-fast",
    ))
}

fn found_crate_path(package: &str, found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(mut name) => {
            // proc-macro-crate uses the package name for an integration
            // test's own library. Our fork keeps its original library name.
            if std::env::var("CARGO_PKG_NAME").as_deref() == Ok(package) {
                match package {
                    "gpui-macos-kit" => name = "gpui_kit".into(),
                    "gpui-macos-component" => name = "gpui_component".into(),
                    _ => {}
                }
            }
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}

/// Resolve the `gpui-component` API exposed to the crate where a macro is
/// expanded, mirroring [`gpui`]: `gpui-kit` consumers reach it as
/// `gpui_kit::component`, standalone consumers as `gpui_component`, and the
/// crate itself as `crate`.
pub(crate) fn component() -> syn::Result<TokenStream> {
    for package in ["gpui-macos-kit", "gpui-kit"] {
        if let Ok(found) = crate_name(package) {
            let kit = found_crate_path(package, found);
            return Ok(quote!(#kit::component));
        }
    }
    if let Ok(found) = crate_name("gpui-macos") {
        let facade = found_crate_path("gpui-macos", found);
        return Ok(quote!(#facade::kit));
    }
    for package in ["gpui-macos-component", "gpui-component"] {
        if let Ok(found) = crate_name(package) {
            return Ok(found_crate_path(package, found));
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "IntoPlot requires gpui-macos, gpui-macos-kit, gpui-macos-component, gpui-kit, or gpui-component",
    ))
}
