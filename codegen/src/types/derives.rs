// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

use crate::CratePath;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    Path,
};

use std::collections::{
    HashMap,
    HashSet,
};

#[derive(Debug, Clone)]
pub struct DerivesRegistry {
    default_derives: Derives,
    specific_type_derives: HashMap<syn::TypePath, Derives>,
}

impl DerivesRegistry {
    /// Creates a new `DeviceRegistry` with the supplied `crate_path`.
    ///
    /// The `crate_path` denotes the `subxt` crate access path in the
    /// generated code.
    pub fn new(crate_path: &CratePath) -> Self {
        Self {
            default_derives: Derives::new(crate_path),
            specific_type_derives: Default::default(),
        }
    }

    /// Insert derives to be applied to all generated types.
    pub fn extend_for_all(&mut self, derives: impl IntoIterator<Item = syn::Path>) {
        self.default_derives.derives.extend(derives)
    }

    /// Insert derives to be applied to a specific generated type.
    pub fn extend_for_type(
        &mut self,
        ty: syn::TypePath,
        derives: impl IntoIterator<Item = syn::Path>,
        crate_path: &CratePath,
    ) {
        let type_derives = self
            .specific_type_derives
            .entry(ty)
            .or_insert_with(|| Derives::new(crate_path));
        type_derives.derives.extend(derives)
    }

    /// Returns the derives to be applied to all generated types.
    pub fn default_derives(&self) -> &Derives {
        &self.default_derives
    }

    /// Resolve the derives for a generated type. Includes:
    ///     - The default derives for all types e.g. `scale::Encode, scale::Decode`
    ///     - Any user-defined derives for all types via `generated_type_derives`
    ///     - Any user-defined derives for this specific type
    pub fn resolve(&self, ty: &syn::TypePath) -> Derives {
        let mut defaults = self.default_derives.derives.clone();
        if let Some(specific) = self.specific_type_derives.get(ty) {
            defaults.extend(specific.derives.iter().cloned());
        }
        Derives { derives: defaults }
    }
}

#[derive(Debug, Clone)]
pub struct Derives {
    derives: HashSet<syn::Path>,
}

impl FromIterator<syn::Path> for Derives {
    fn from_iter<T: IntoIterator<Item = Path>>(iter: T) -> Self {
        let derives = iter.into_iter().collect();
        Self { derives }
    }
}

impl Derives {
    /// Creates a new instance of `Derives` with the `crate_path` prepended
    /// to the set of default derives that reside in `subxt`.
    pub fn new(crate_path: &CratePath) -> Self {
        let mut derives = HashSet::new();
        derives.insert(syn::parse_quote!(#crate_path::ext::codec::Encode));
        derives.insert(syn::parse_quote!(#crate_path::ext::codec::Decode));
        derives.insert(syn::parse_quote!(Debug));
        Self { derives }
    }

    /// Add `#crate_path::ext::codec::CompactAs` to the derives.
    pub fn insert_codec_compact_as(&mut self, crate_path: &CratePath) {
        self.insert(parse_quote!(#crate_path::ext::codec::CompactAs));
    }

    pub fn append(&mut self, derives: impl Iterator<Item = syn::Path>) {
        for derive in derives {
            self.insert(derive)
        }
    }

    pub fn insert(&mut self, derive: syn::Path) {
        self.derives.insert(derive);
    }
}

impl quote::ToTokens for Derives {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if !self.derives.is_empty() {
            let mut sorted = self.derives.iter().cloned().collect::<Vec<_>>();
            sorted.sort_by(|a, b| {
                quote::quote!(#a)
                    .to_string()
                    .cmp(&quote::quote!(#b).to_string())
            });
            let derives: Punctuated<syn::Path, syn::Token![,]> =
                sorted.iter().cloned().collect();
            tokens.extend(quote::quote! {
                #[derive(#derives)]
            })
        }
    }
}
