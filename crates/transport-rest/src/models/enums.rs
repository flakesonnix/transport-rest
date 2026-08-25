//! String enums with a forward-compatible fallback variant.
//!
//! transport.rest instances occasionally introduce new values (e.g. new
//! transport `mode`s or product keys). Every enum here therefore has an
//! [`Enum::Other`](Mode::Other)-style variant that captures unknown wire
//! values losslessly instead of failing deserialization.

/// Generate a string enum with an open fallback.
macro_rules! open_string_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$vmeta:meta])* $variant:ident => $value:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
            /// A value this library does not know yet.
            ///
            /// Preserved verbatim so upgrades never lose data; match on it to
            /// treat unknown values gracefully.
            Other(String),
        }

        impl $name {
            /// All values known at compile time.
            pub const KNOWN: &'static [&'static str] = &[$($value),+];

            /// The canonical wire representation.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $value,)+
                    Self::Other(s) => s.as_str(),
                }
            }

            /// Parse without failing on unknown values.
            pub fn from_str_lossy(s: &str) -> Self {
                match s {
                    $($value => Self::$variant,)+
                    other => Self::Other(other.to_owned()),
                }
            }

            /// True if this is the fallback for an unknown value.
            pub fn is_other(&self) -> bool {
                matches!(self, Self::Other(_))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::from_str_lossy(s)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::from_str_lossy(&s)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                Ok(Self::from_str_lossy(&s))
            }
        }
    };
}

open_string_enum! {
    /// Means of transport mode (FPTF).
    Mode {
        /// Aircraft.
        Aircraft => "aircraft",
        /// Bicycle.
        Bicycle => "bicycle",
        /// Bus.
        Bus => "bus",
        /// Car.
        Car => "car",
        /// Gondola / aerial lift.
        Gondola => "gondola",
        /// Taxi.
        Taxi => "taxi",
        /// Train (any kind).
        Train => "train",
        /// Walking.
        Walking => "walking",
        /// Watercraft.
        Watercraft => "watercraft",
    }
}

open_string_enum! {
    /// Reliability class of a prognosis.
    PrognosisType {
        /// The prognosis was calculated from realtime data.
        Calculated => "calculated",
        /// The value is a rough prognosis.
        Prognosed => "prognosed",
    }
}

open_string_enum! {
    /// Kind of remark (FPTF hint/status/warning merged).
    RemarkKind {
        /// Generic hint shown in apps.
        Hint => "hint",
        /// Status message.
        Status => "status",
        /// Disruption warning.
        Warning => "warning",
        /// Foreign ID metadata remark.
        ForeignId => "foreign-id",
        /// Local fare zone information.
        LocalFareZone => "local-fare-zone",
        /// DELFI Haltestellen-ID of a stop.
        StopDhid => "stop-dhid",
        /// Website of a stop.
        StopWebsite => "stop-website",
        /// Transit authority operating the line/stop.
        TransitAuthority => "transit-authority",
    }
}

open_string_enum! {
    /// db-vendo backend profile of the DB instance.
    ///
    /// Different profiles return different amounts of detail and have
    /// different quotas; see docs/API_ANALYSIS.md.
    DbProfile {
        /// Default profile (`dbnav`).
        Dbnav => "dbnav",
        /// Classic profile (`db`).
        Db => "db",
        /// Web profile (`dbweb`); supports `direction` filter.
        Dbweb => "dbweb",
    }
}
