extern crate proc_macro;

use proc_macro::TokenStream;

mod codegen;
mod preset;

#[proc_macro_derive(Preset, attributes(preset, preset_field))]
pub fn derive_preset(input: TokenStream) -> TokenStream {
    preset::derive_preset_impl(input)
}
