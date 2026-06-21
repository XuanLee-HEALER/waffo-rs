//! Deserialization helpers.

use serde::{Deserialize, Deserializer};

/// Deserialize a value, treating an explicit JSON `null` as `T::default()`.
///
/// `#[serde(default)]` only covers *absent* fields; the Waffo API also returns
/// an explicit `null` for some empty collection fields, which would otherwise
/// fail to deserialize (`invalid type: null, expected a map/sequence`). Pair
/// this with `default`:
///
/// ```ignore
/// #[serde(default, deserialize_with = "crate::common::de::null_as_default")]
/// pub limits: std::collections::HashMap<String, String>,
/// ```
pub(crate) fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
