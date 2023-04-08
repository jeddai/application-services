// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod defaults;
mod enrollment;
mod evaluator;
mod matcher;
mod sampling;
mod strings;
mod targeting;

pub mod error;
pub mod schema;
pub mod versioning;

pub use enrollment::{EnrolledFeature, EnrollmentChangeEvent, EnrollmentChangeEventType};
pub use error::{NimbusError, Result};
#[cfg(debug_assertions)]
pub use evaluator::evaluate_enrollment;
pub use matcher::AppContext;
pub use schema::*;
pub use targeting::NimbusTargetingHelper;

use serde_json::Value;

cfg_if::cfg_if! {
    if #[cfg(feature = "stateful")] {
        mod behavior;
        mod client;
        mod config;
        mod dbcache;
        mod updating;

        pub mod nimbus_client;
        pub mod persistence;

        pub use crate::nimbus_client::*;
        pub use config::RemoteSettingsConfig;
    } else {
        pub mod stateless {
            pub mod cirrus_client;
        }

        pub use crate::stateless::cirrus_client::*;
    }
}

// Exposed for Example only
pub use evaluator::TargetingAttributes;

#[cfg(test)]
mod tests;

impl UniffiCustomTypeConverter for JsonObject {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        let json: Value = serde_json::from_str(&val)?;

        match json.as_object() {
            Some(obj) => Ok(obj.clone()),
            _ => Err(anyhow::anyhow!(
                "Unexpected JSON-non-object in the bagging area"
            )),
        }
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        Value::Object(obj).to_string()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "stateful-uniffi-bindings")] {
        include!(concat!(env!("OUT_DIR"), "/nimbus.uniffi.rs"));
    } else {
        pub use enrollment::{
            EnrolledFeatureConfig, ExperimentEnrollment, EnrolledReason, DisqualifiedReason,
            NotEnrolledReason, EnrollmentStatus
        };
        use std::collections::HashSet;
        use std::str::FromStr;
        use uuid::Uuid;

        impl UniffiCustomTypeConverter for Value {
            type Builtin = String;

            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let json: Value = serde_json::from_str(&val)?;
                Ok(json)
            }

            fn from_custom(val: Self) -> Self::Builtin {
                val.to_string()
            }
        }

        impl UniffiCustomTypeConverter for Uuid {
            type Builtin = String;

            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let uuid: Uuid = Uuid::from_str(&val)?;
                Ok(uuid)
            }

            fn from_custom(val: Self) -> Self::Builtin {
                val.to_string()
            }
        }

        pub type StringHashSet = HashSet<String>;

        impl UniffiCustomTypeConverter for StringHashSet {
            type Builtin = String;

            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let set: Value = serde_json::from_str(&val)?;

                match set.as_array() {
                    Some(obj) => Ok(HashSet::from_iter(obj.clone().iter().map(|x| x.as_str().unwrap().to_string()))),
                    _ => Err(anyhow::anyhow!(
                        "Unexpected JSON-non-array in the bagging area"
                    )),
                }
            }

            fn from_custom(val: Self) -> Self::Builtin {
                Value::Array(Vec::from_iter(val).iter().map(|x| Value::String(x.to_owned())).collect()).to_string()
            }
        }

        include!(concat!(env!("OUT_DIR"), "/cirrus.uniffi.rs"));
    }
}
