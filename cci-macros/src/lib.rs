extern crate proc_macro;

use proc_macro::TokenStream;

mod codegen;
mod preset;
mod preset_enum;

#[proc_macro_derive(Preset, attributes(preset, preset_field))]
pub fn derive_preset(input: TokenStream) -> TokenStream {
    preset::derive_preset_impl(input)
}

#[proc_macro_derive(PresetEnum, attributes(preset_enum, preset_variant))]
pub fn derive_preset_enum(input: TokenStream) -> TokenStream {
    preset_enum::derive_preset_enum_impl(input)
}
