use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::codegen::{generate_conversions, generate_editor_preset_impl, generate_ron_type};

/// Struct-level attributes for #[preset(...)]
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(preset), supports(struct_named))]
pub struct PresetOpts {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<(), PresetFieldOpts>,

    /// Preset ID (e.g., "rust", "python-app")
    pub id: String,

    /// Display name (e.g., "Rust", "Python App")
    pub name: String,

    /// User-facing description
    pub description: String,

    /// ProjectType pattern for matches_project() (e.g., "RustBinary | RustLibrary")
    #[darling(default)]
    pub matches: Option<String>,
}

/// Field-level attributes for #[preset_field(...)]
#[derive(Debug, Clone, FromField)]
#[darling(attributes(preset_field))]
pub struct PresetFieldOpts {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    /// Default value expression
    #[darling(default)]
    pub default: Option<String>,

    /// Hide from TUI
    #[darling(default)]
    pub hidden: bool,

    /// Display name in TUI
    #[darling(default)]
    pub display: Option<String>,

    /// User-facing description
    #[darling(default)]
    pub description: Option<String>,

    /// Feature group ID
    #[darling(default)]
    pub feature: Option<String>,

    /// Feature group display name
    #[darling(default)]
    pub feature_display: Option<String>,
}

pub fn derive_preset_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match PresetOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    // Extract fields before consuming opts
    let fields: Vec<_> = opts.data.clone().take_struct().unwrap().fields;
    let preset_ident = &opts.ident;

    // Generate the RON config struct
    let ron_type = generate_ron_type(&opts.ident, &opts.id, &fields);

    // Generate conversion methods
    let conversions = generate_conversions(&opts.ident, &opts.id, &fields);

    // Generate EditorPreset trait implementation
    let editor_preset = generate_editor_preset_impl(&opts, &fields);

    // Generate default() method using field defaults
    let default_fields = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let default_expr = field
            .default
            .as_ref()
            .map(|s| s.parse::<proc_macro2::TokenStream>().unwrap())
            .unwrap_or_else(|| quote::quote! { Default::default() });
        quote::quote! {
            #field_ident: #default_expr
        }
    });

    let default_impl = quote::quote! {
        impl #preset_ident {
            pub fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    };

    let expanded = quote! {
        #ron_type
        #conversions
        #editor_preset
        #default_impl
    };

    TokenStream::from(expanded)
}
