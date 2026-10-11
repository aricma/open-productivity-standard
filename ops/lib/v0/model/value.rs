//! The metadata value types: the six shapes the standard carries (rule
//! 7), in ops-lib's own terms so `serde_json` stays an implementation
//! detail.
//!
//! A value is opaque data. The model never interprets it — no charset, no
//! case rule, no language — so these types only keep the shape, and are
//! (de)serialized through `serde_json` internally.

use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;

/// A metadata map: keys to values, ordered by key.
#[derive(Clone, Default, PartialEq, Deserialize)]
pub struct Metadata(BTreeMap<String, MetadataValue>);

/// One metadata value: a string, number, boolean, `null`, list, or map.
#[derive(Clone, PartialEq)]
pub enum MetadataValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    List(Vec<MetadataValue>),
    Map(Metadata),
}

/// A number, kept in the representation that holds `i64`, `u64`, and
/// `f64` exactly.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Number(serde_json::Number);

impl Metadata {
    /// A map read from JSON. Internal: a caller outside the crate bridges
    /// through serde's `Deserialize` instead.
    pub(crate) fn from_json_map(value: serde_json::Map<String, serde_json::Value>) -> Metadata {
        value
            .into_iter()
            .map(|(key, value)| (key, MetadataValue::from_json(value)))
            .collect()
    }

    pub fn new() -> Metadata {
        Metadata::default()
    }

    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.0.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn insert(&mut self, key: String, value: MetadataValue) -> Option<MetadataValue> {
        self.0.insert(key, value)
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&String, &mut MetadataValue) -> bool) {
        self.0.retain(keep);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &MetadataValue)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl MetadataValue {
    /// Whether this value carries nothing: `null`, or an empty string, list, or
    /// map. A reader reads such a value as absent, so a writer leaves it out.
    /// (rule 6, RFC-027)
    pub(crate) fn carries_nothing(&self) -> bool {
        match self {
            MetadataValue::Null => true,
            MetadataValue::String(text) => text.is_empty(),
            MetadataValue::List(items) => items.is_empty(),
            MetadataValue::Map(map) => map.is_empty(),
            MetadataValue::Bool(_) | MetadataValue::Number(_) => false,
        }
    }

    /// A value read from JSON. Internal: a caller outside the crate
    /// bridges through serde's `Deserialize` instead.
    pub(crate) fn from_json(value: serde_json::Value) -> MetadataValue {
        match value {
            serde_json::Value::Null => MetadataValue::Null,
            serde_json::Value::Bool(v) => MetadataValue::Bool(v),
            serde_json::Value::Number(v) => MetadataValue::Number(Number(v)),
            serde_json::Value::String(v) => MetadataValue::String(v),
            serde_json::Value::Array(v) => {
                MetadataValue::List(v.into_iter().map(MetadataValue::from_json).collect())
            }
            serde_json::Value::Object(v) => MetadataValue::Map(Metadata::from_json_map(v)),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, MetadataValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MetadataValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self {
            MetadataValue::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            MetadataValue::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[MetadataValue]> {
        match self {
            MetadataValue::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&Metadata> {
        match self {
            MetadataValue::Map(v) => Some(v),
            _ => None,
        }
    }
}

impl Number {
    pub fn from_f64(value: f64) -> Option<Number> {
        serde_json::Number::from_f64(value).map(Number)
    }

    pub fn is_i64(&self) -> bool {
        self.0.is_i64()
    }

    pub fn is_u64(&self) -> bool {
        self.0.is_u64()
    }

    pub fn is_f64(&self) -> bool {
        self.0.is_f64()
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.0.as_i64()
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Number {
        Number(value.into())
    }
}

impl From<u64> for Number {
    fn from(value: u64) -> Number {
        Number(value.into())
    }
}

impl FromIterator<(String, MetadataValue)> for Metadata {
    fn from_iter<T: IntoIterator<Item = (String, MetadataValue)>>(iter: T) -> Metadata {
        Metadata(iter.into_iter().collect())
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

impl fmt::Debug for MetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataValue::Null => f.write_str("null"),
            MetadataValue::Bool(v) => v.fmt(f),
            MetadataValue::Number(v) => v.fmt(f),
            MetadataValue::String(v) => v.fmt(f),
            MetadataValue::List(v) => v.fmt(f),
            MetadataValue::Map(v) => v.fmt(f),
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Serde predicate: a string field that carries nothing is left out, since a
/// reader reads it as absent.
pub(crate) fn absent_string(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(str::is_empty)
}

/// Serde predicate: metadata that carries nothing is left out, like a value
/// that carries nothing.
pub(crate) fn absent_metadata(value: &Option<Metadata>) -> bool {
    value.as_ref().is_none_or(Metadata::is_empty)
}

/// A map is written without the values that carry nothing: the reader reads
/// them as absent, so writing one would not survive a round trip. (RFC-027)
impl Serialize for Metadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let written = self
            .0
            .values()
            .filter(|value| !value.carries_nothing())
            .count();
        let mut map = serializer.serialize_map(Some(written))?;
        for (key, value) in &self.0 {
            if !value.carries_nothing() {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

impl Serialize for MetadataValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MetadataValue::Null => serializer.serialize_unit(),
            MetadataValue::Bool(v) => serializer.serialize_bool(*v),
            MetadataValue::Number(v) => v.serialize(serializer),
            MetadataValue::String(v) => serializer.serialize_str(v),
            MetadataValue::List(v) => v.serialize(serializer),
            MetadataValue::Map(v) => v.serialize(serializer),
        }
    }
}

/// A metadata value is any JSON value, so the reader takes whatever the
/// format hands it and keeps the shape.
impl<'de> Deserialize<'de> for MetadataValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde_json::Value::deserialize(deserializer).map(MetadataValue::from_json)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn every_shape_round_trips_through_json() {
        // No `null` entry: a value that carries nothing is not written (see
        // `serializing_metadata_leaves_out_what_carries_nothing`). A `null`
        // nested inside a value is data, and stays.
        let json = serde_json::json!({
            "bool": true,
            "int": -9007199254740993_i64,
            "uint": 18446744073709551615_u64,
            "float": 1.5,
            "string": "x",
            "list": [1, "two", null],
            "map": { "nested": [{ "deep": false }] },
        });
        let value: Metadata = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&value).unwrap(), json);
    }

    /// `f64` cannot hold either of these exactly, so the number type has
    /// to keep the integer representation.
    #[test]
    fn integers_keep_their_exact_value() {
        let value: Metadata = serde_json::from_value(serde_json::json!({
            "big": 9007199254740993_u64,
            "small": -9007199254740993_i64,
        }))
        .unwrap();
        let big = value.get("big").and_then(MetadataValue::as_number).unwrap();
        let small = value
            .get("small")
            .and_then(MetadataValue::as_number)
            .unwrap();
        assert_eq!(big.as_u64(), Some(9007199254740993));
        assert_eq!(small.as_i64(), Some(-9007199254740993));
    }

    /// A value that carries nothing is not part of a document: the writer
    /// leaves it out, since a reader would drop it anyway.
    #[test]
    fn serializing_metadata_leaves_out_what_carries_nothing() {
        let mut metadata = Metadata::new();
        metadata.insert("priority".into(), MetadataValue::String("high".into()));
        metadata.insert("blank".into(), MetadataValue::String(String::new()));
        metadata.insert("nothing".into(), MetadataValue::Null);
        metadata.insert("none".into(), MetadataValue::List(Vec::new()));
        metadata.insert("void".into(), MetadataValue::Map(Metadata::new()));
        assert_eq!(
            serde_json::to_value(&metadata).unwrap(),
            serde_json::json!({ "priority": "high" })
        );
    }

    #[test]
    fn accessors_read_the_shape() {
        assert!(MetadataValue::Null.is_null());
        assert_eq!(MetadataValue::Bool(true).as_bool(), Some(true));
        assert_eq!(MetadataValue::String("x".into()).as_str(), Some("x"));
        assert_eq!(MetadataValue::List(vec![]).as_list(), Some(&[][..]));
        assert!(MetadataValue::Map(Metadata::new()).as_map().is_some());
        assert_eq!(MetadataValue::Null.as_str(), None);
    }
}
