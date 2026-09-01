use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::codegen::{generate_conversions, generate_editor_preset_impl};

/// Struct-level attributes for #[preset(...)]
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(preset), forward_attrs(doc), supports(struct_named))]
pub struct PresetOpts {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<(), PresetFieldOpts>,
    pub attrs: Vec<syn::Attribute>,

    /// Preset category (e.g., "Languages", "Packaging", "Documentation")
    pub category: String,
}

impl PresetOpts {
    /// Extract the doc comment from struct attributes
    pub fn doc_comment(&self) -> String {
        extract_doc_comment(&self.attrs)
    }

    /// Derive a display name from the struct ident by splitting CamelCase
    /// e.g., `Rust` → `"Rust"`, `PythonApp` → `"Python App"`
    pub fn display_name(&self) -> String {
        camel_case_to_display(&self.ident.to_string())
    }

    /// Derive the preset ID from the struct ident (lowercase of struct name)
    /// e.g., `Rust` → `"Rust"`, `PythonApp` → `"PythonApp"`
    pub fn preset_id(&self) -> String {
        self.ident.to_string()
    }

    /// Derive the match prefix (first CamelCase word) for project type detection
    /// e.g., `Rust` → `"Rust"`, `PythonApp` → `"Python"`, `Docker` → `"Docker"`
    pub fn match_prefix(&self) -> String {
        first_camel_word(&self.ident.to_string())
    }
}

/// Field-level attributes for #[preset_field(...)]
#[derive(Debug, Clone, FromField)]
#[darling(attributes(preset_field), forward_attrs(doc))]
pub struct PresetFieldOpts {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,

    /// Hide from TUI
    #[darling(default)]
    pub hidden: bool,

    /// Display name in TUI
    #[darling(default)]
    pub display: Option<String>,
}

impl PresetFieldOpts {
    /// Extract the doc comment from field attributes
    pub fn doc_comment(&self) -> String {
        extract_doc_comment(&self.attrs)
    }
}

/// Extract doc comment text from `#[doc = "..."]` attributes
fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Split a CamelCase identifier into space-separated words
/// e.g., `"PythonApp"` → `"Python App"`, `"Rust"` → `"Rust"`
fn camel_case_to_display(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}

/// Extract the first CamelCase word from an identifier
/// e.g., `"PythonApp"` → `"Python"`, `"Rust"` → `"Rust"`, `"GoApp"` → `"Go"`
fn first_camel_word(s: &str) -> String {
    let mut end = s.len();
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            end = i;
            break;
        }
    }
    s[..end].to_string()
}

pub fn derive_preset_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match PresetOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    // Extract fields before consuming opts
    let fields: Vec<_> = opts.data.clone().take_struct().unwrap().fields;
    let preset_id = opts.preset_id();

    // Generate conversion methods
    let conversions = generate_conversions(&opts.ident, &preset_id, &fields);

    // Generate EditorPreset trait implementation
    let editor_preset = generate_editor_preset_impl(&opts, &fields);

    let expanded = quote! {
        #conversions
        #editor_preset
    };

    TokenStream::from(expanded)
}
