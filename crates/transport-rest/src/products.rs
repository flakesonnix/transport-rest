//! Product filter for departure boards and journey searches.
//!
//! transport.rest instances expose product filters as flat boolean query
//! parameters whose keys depend on the provider profile. Unset keys are
//! omitted (the server defaults to "include everything").

/// Selection of transport products to include in a query.
///
/// Only explicitly set products are sent; the server default includes all
/// products. Known keys get typed setters, unknown provider-specific keys can
/// be set via [`ProductSelection::set`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProductSelection {
    entries: Vec<(String, bool)>,
}

macro_rules! known_products {
    ($($method:ident => $key:literal),+ $(,)?) => {
        impl ProductSelection {
            $(
                /// Include or exclude this product.
                #[doc = concat!("Corresponds to the `", $key, "` query parameter.")]
                pub fn $method(mut self, enabled: bool) -> Self {
                    self.set_internal($key.to_owned(), enabled);
                    self
                }
            )+
        }

        impl ProductSelection {
            /// All product keys commonly supported across providers.
            pub const KNOWN_KEYS: &'static [&'static str] = &[$($key),+];
        }
    };
}

known_products! {
    national_express => "nationalExpress",
    national => "national",
    regional_express => "regionalExpress",
    regional => "regional",
    suburban => "suburban",
    subway => "subway",
    tram => "tram",
    bus => "bus",
    ferry => "ferry",
    taxi => "taxi",
    express => "express",
}

impl ProductSelection {
    /// Set an arbitrary (possibly provider-specific) product key.
    pub fn set(mut self, key: impl Into<String>, enabled: bool) -> Self {
        self.set_internal(key.into(), enabled);
        self
    }

    fn set_internal(&mut self, key: String, enabled: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            entry.1 = enabled;
        } else {
            self.entries.push((key, enabled));
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn encode(&self, q: &mut crate::request::Query) {
        for (key, enabled) in &self.entries {
            q.push(key, (*enabled).to_string());
        }
    }
}
