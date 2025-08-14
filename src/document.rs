#![allow(clippy::new_ret_no_self)]
#![allow(clippy::wrong_self_convention)]

use napi::{bindgen_prelude::*, Error, JsNumber, Result, Status, ValueType};
use napi_derive::napi;

use tantivy::{self as tv, schema::document::OwnedValue as Value};

use crate::{facet::Facet, schema::Schema, to_napi_error};
use chrono;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::{
  collections::BTreeMap,
  fmt,
  net::{IpAddr, Ipv6Addr},
  str::FromStr,
};

// Helper function to convert JS object to JSON value
fn js_object_to_json_value(obj: Object) -> Result<serde_json::Value> {
  let keys = obj.get_property_names()?;
  let mut map = serde_json::Map::new();

  for i in 0..keys.get_array_length()? {
    let key: String = keys.get_element(i)?;
    let value_js: Unknown = obj.get_named_property(&key)?;

    let json_value = match value_js.get_type()? {
      ValueType::String => {
        let s: String = value_js
          .coerce_to_string()?
          .into_utf8()?
          .as_str()?
          .to_string();
        serde_json::Value::String(s)
      }
      ValueType::Number => {
        let n: f64 = value_js.coerce_to_number()?.get_double()?;
        serde_json::Value::Number(
          serde_json::Number::from_f64(n)
            .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid number".to_string()))?,
        )
      }
      ValueType::Boolean => {
        let b: bool = value_js.coerce_to_bool()?;
        serde_json::Value::Bool(b)
      }
      ValueType::Object => {
        let inner_obj: Object = unsafe { value_js.cast()? };
        if value_js.is_array()? {
          let len = inner_obj.get_array_length()?;
          let mut arr = Vec::new();
          for j in 0..len {
            let elem: Unknown = inner_obj.get_element(j)?;
            arr.push(js_unknown_to_json_value(elem)?);
          }
          serde_json::Value::Array(arr)
        } else {
          js_object_to_json_value(inner_obj)?
        }
      }
      ValueType::Null => serde_json::Value::Null,
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          "Unsupported value type".to_string(),
        ))
      }
    };

    map.insert(key, json_value);
  }

  Ok(serde_json::Value::Object(map))
}

fn js_unknown_to_json_value(value: Unknown) -> Result<serde_json::Value> {
  match value.get_type()? {
    ValueType::String => {
      let s: String = value.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
      Ok(serde_json::Value::String(s))
    }
    ValueType::Number => {
      let n: f64 = value.coerce_to_number()?.get_double()?;
      Ok(serde_json::Value::Number(
        serde_json::Number::from_f64(n)
          .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid number".to_string()))?,
      ))
    }
    ValueType::Boolean => {
      let b: bool = value.coerce_to_bool()?;
      Ok(serde_json::Value::Bool(b))
    }
    ValueType::Object => {
      let obj: Object = unsafe { value.cast()? };
      js_object_to_json_value(obj)
    }
    ValueType::Null => Ok(serde_json::Value::Null),
    _ => Err(Error::new(
      Status::InvalidArg,
      "Unsupported value type".to_string(),
    )),
  }
}

// Simplified helper function for value extraction (similar to Python version)
pub(crate) fn extract_value(value: &Unknown) -> Result<Value> {
  match value.get_type()? {
    ValueType::String => {
      let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
      Ok(Value::Str(s))
    }
    ValueType::Boolean => {
      let b = value.coerce_to_bool()?;
      Ok(Value::Bool(b))
    }
    ValueType::Number => {
      let n = value.coerce_to_number()?.get_double()?;
      // Smart integer detection
      if n.fract() == 0.0 && n.is_finite() {
        if n >= 0.0 && n <= u64::MAX as f64 {
          Ok(Value::U64(n as u64))
        } else {
          Ok(Value::I64(n as i64))
        }
      } else {
        Ok(Value::F64(n))
      }
    }
    ValueType::Object => {
      if value.is_buffer()? {
        // Handle Buffer objects as bytes
        let obj = value.coerce_to_object()?;
        let length = obj.get_named_property::<JsNumber>("length")?;
        let len = length.get_uint32()?;
        let mut bytes = Vec::new();
        for i in 0..len {
          let item = obj.get_element::<JsNumber>(i)?;
          let byte_val = item.get_uint32()?;
          if byte_val <= 255 {
            bytes.push(byte_val as u8);
          } else {
            return Err(Error::new(Status::InvalidArg, "Byte value out of range"));
          }
        }
        Ok(Value::Bytes(bytes))
      } else if value.is_array()? {
        // Handle arrays
        let obj: Object = unsafe { value.cast()? };
        let len = obj.get_array_length()?;
        let mut values = Vec::new();
        for i in 0..len {
          let item: Unknown = obj.get_element(i)?;
          values.push(extract_value(&item)?);
        }
        Ok(Value::Array(values))
      } else {
        // Handle objects - directly use Value::from(serde_json::Value)
        let obj: Object = unsafe { value.cast()? };
        let json_value = js_object_to_json_value(obj)?;
        Ok(Value::from(json_value))
      }
    }
    _ => Err(Error::new(
      Status::InvalidArg,
      format!("Unsupported value type: {:?}", value.get_type()),
    )),
  }
}

// Simplified schema-aware value extraction (similar to Python)
pub(crate) fn extract_value_for_type(
  value: &Unknown,
  tv_type: tv::schema::Type,
  field_name: &str,
) -> Result<Value> {
  let error_msg = |type_name: &str| {
    format!(
      "Expected {} type for field {}, got unexpected value",
      type_name, field_name
    )
  };

  match tv_type {
    tv::schema::Type::Str => {
      let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
      Ok(Value::Str(s))
    }
    tv::schema::Type::U64 => {
      // Reject strings but allow number coercion
      if matches!(value.get_type()?, ValueType::String) {
        return Err(Error::new(Status::InvalidArg, error_msg("U64")));
      }
      let n = value.coerce_to_number()?.get_double()?;
      Ok(Value::U64(n.abs() as u64))
    }
    tv::schema::Type::I64 => {
      if matches!(value.get_type()?, ValueType::String) {
        return Err(Error::new(Status::InvalidArg, error_msg("I64")));
      }
      let n = value.coerce_to_number()?.get_double()?;
      Ok(Value::I64(n as i64))
    }
    tv::schema::Type::F64 => {
      if matches!(value.get_type()?, ValueType::String) {
        return Err(Error::new(Status::InvalidArg, error_msg("F64")));
      }
      let n = value.coerce_to_number()?.get_double()?;
      Ok(Value::F64(n))
    }
    tv::schema::Type::Bool => {
      let b = value.coerce_to_bool()?;
      Ok(Value::Bool(b))
    }
    tv::schema::Type::Date => {
      match value.get_type()? {
        ValueType::Number => {
          let timestamp = value.coerce_to_number()?.get_int64()?;
          // JavaScript timestamps are in milliseconds
          Ok(Value::Date(tv::DateTime::from_timestamp_secs(
            timestamp / 1000,
          )))
        }
        ValueType::String => {
          // Handle ISO date strings
          let date_str = value.coerce_to_string()?.into_utf8()?.into_owned()?;
          if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&date_str) {
            Ok(Value::Date(tv::DateTime::from_timestamp_secs(
              dt.timestamp(),
            )))
          } else {
            Err(Error::new(
              Status::InvalidArg,
              format!("Invalid ISO date string: {}", date_str),
            ))
          }
        }
        _ => Err(Error::new(Status::InvalidArg, error_msg("DateTime"))),
      }
    }
    tv::schema::Type::Facet => {
      let facet_str = value.coerce_to_string()?.into_utf8()?.into_owned()?;
      let facet = tv::schema::Facet::from_text(&facet_str)
        .map_err(|_| Error::new(Status::InvalidArg, error_msg("Facet")))?;
      Ok(Value::Facet(facet))
    }
    tv::schema::Type::Bytes => {
      // Node.js: Only accept Buffer objects, not arrays
      if !value.is_buffer()? {
        return Err(Error::new(
          Status::InvalidArg,
          "Expected Buffer for bytes field",
        ));
      }
      extract_value(value) // Reuse the simplified extraction
    }
    tv::schema::Type::IpAddr => {
      let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
      let ip_addr = IpAddr::from_str(&s).map_err(to_napi_error)?;
      let ipv6_addr = match ip_addr {
        IpAddr::V4(addr) => addr.to_ipv6_mapped(),
        IpAddr::V6(addr) => addr,
      };
      Ok(Value::IpAddr(ipv6_addr))
    }
    tv::schema::Type::Json => {
      // Direct conversion using Value::from(serde_json::Value)
      let obj: Object = unsafe { value.cast()? };
      let json_value = js_object_to_json_value(obj)?;
      Ok(Value::from(json_value))
    }
  }
}

fn extract_value_single_or_list(value: &Unknown) -> Result<Vec<Value>> {
  // Check if it's a string first, since strings are array-like in JavaScript
  if matches!(value.get_type()?, ValueType::String) {
    return Ok(vec![extract_value(value)?]);
  }

  // Try to treat as array first
  if value.is_array()? {
    let obj: Object = unsafe { value.cast()? };
    let len = obj.get_array_length()?;
    let mut values = Vec::new();
    for i in 0..len {
      let item: Unknown = obj.get_element(i)?;
      values.push(extract_value(&item)?);
    }
    return Ok(values);
  }
  // Treat as single value
  Ok(vec![extract_value(value)?])
}

fn extract_value_single_or_list_for_type(
  value: &Unknown,
  field_type: &tv::schema::FieldType,
  field_name: &str,
) -> Result<Vec<Value>> {
  // Check if it's a string first, since strings are array-like in JavaScript
  if matches!(value.get_type()?, ValueType::String) {
    return Ok(vec![extract_value_for_type(
      value,
      field_type.value_type(),
      field_name,
    )?]);
  }

  // Try to treat as array first
  if value.is_array()? {
    let obj: Object = unsafe { value.cast()? };
    let len = obj.get_array_length()?;

    // For bytes fields, don't process arrays as single entries since Node.js version doesn't support them
    if field_type.value_type() == tv::schema::Type::Bytes {
      return Err(Error::new(
        Status::InvalidArg,
        "Expected Buffer for bytes field",
      ));
    }

    let mut values = Vec::new();
    for i in 0..len {
      let item: Unknown = obj.get_element(i)?;
      values.push(extract_value_for_type(
        &item,
        field_type.value_type(),
        field_name,
      )?);
    }
    return Ok(values);
  }
  // Treat as single value
  Ok(vec![extract_value_for_type(
    value,
    field_type.value_type(),
    field_name,
  )?])
}

fn value_to_js(env: Env, value: &Value) -> Result<Unknown> {
  Ok(match value {
    Value::Str(text) => env.to_js_value(&text.as_str())?,
    Value::U64(num) => env.to_js_value(&(*num as f64))?,
    Value::I64(num) => env.to_js_value(&(*num as f64))?,
    Value::F64(num) => env.to_js_value(num)?,
    Value::Bytes(b) => env.to_js_value(&b.as_slice())?,
    Value::PreTokStr(_pretoken) => env.to_js_value(&())?,
    Value::Date(d) => {
      let timestamp = d.into_timestamp_secs();
      env.to_js_value(&(timestamp as f64 * 1000.0))?
    }
    Value::Facet(f) => env.to_js_value(&f.to_string())?,
    Value::Array(arr) => {
      let vec: Vec<serde_json::Value> = arr.iter().map(|v| value_to_serde_json(v)).collect();
      env.to_js_value(&vec)?
    }
    Value::Object(obj) => {
      let map: std::collections::HashMap<String, serde_json::Value> = obj
        .iter()
        .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
        .collect();
      env.to_js_value(&map)?
    }
    Value::Bool(b) => env.to_js_value(b)?,
    Value::IpAddr(i) => {
      // Convert IPv4-mapped IPv6 addresses back to IPv4 format for display
      let addr_str = if let Some(ipv4) = i.to_ipv4_mapped() {
        ipv4.to_string()
      } else {
        i.to_string()
      };
      env.to_js_value(&addr_str)?
    }
    Value::Null => env.to_js_value(&())?,
  })
}

fn value_to_serde_json(value: &Value) -> serde_json::Value {
  match value {
    Value::Str(s) => serde_json::Value::String(s.clone()),
    Value::U64(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
    Value::I64(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
    Value::F64(n) => serde_json::Value::Number(
      serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
    ),
    Value::Bool(b) => serde_json::Value::Bool(*b),
    Value::Date(d) => {
      let timestamp = d.into_timestamp_secs();
      serde_json::Value::Number(serde_json::Number::from(timestamp))
    }
    Value::Facet(f) => serde_json::Value::String(f.to_string()),
    Value::Bytes(b) => serde_json::Value::Array(
      b.iter()
        .map(|&byte| serde_json::Value::Number(serde_json::Number::from(byte)))
        .collect(),
    ),
    Value::IpAddr(ip) => {
      // Convert IPv4-mapped IPv6 addresses back to IPv4 format for display
      let addr_str = if let Some(ipv4) = ip.to_ipv4_mapped() {
        ipv4.to_string()
      } else {
        ip.to_string()
      };
      serde_json::Value::String(addr_str)
    }
    Value::Object(obj) => {
      let map: serde_json::Map<String, serde_json::Value> = obj
        .iter()
        .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
        .collect();
      serde_json::Value::Object(map)
    }
    Value::Array(arr) => {
      let vec: Vec<serde_json::Value> = arr.iter().map(|v| value_to_serde_json(v)).collect();
      serde_json::Value::Array(vec)
    }
    _ => serde_json::Value::Null,
  }
}

fn value_to_string(value: &Value) -> String {
  match value {
    Value::Null => format!("{:?}", value),
    Value::Str(text) => text.clone(),
    Value::U64(num) => format!("{num}"),
    Value::I64(num) => format!("{num}"),
    Value::F64(num) => format!("{num}"),
    Value::Bytes(bytes) => format!("{bytes:?}"),
    Value::Date(d) => format!("{d:?}"),
    Value::Facet(facet) => facet.to_string(),
    Value::PreTokStr(_pretok) => {
      // TODO implement me
      "PreTokStr(...)".to_string()
    }
    Value::Array(arr) => {
      let inner: Vec<_> = arr.iter().map(value_to_string).collect();
      format!("{inner:?}")
    }
    Value::Object(json_object) => serde_json::to_string(&json_object).unwrap(),
    Value::Bool(b) => format!("{b}"),
    Value::IpAddr(i) => format!("{}", *i),
  }
}

/// Serializes a [`tv::DateTime`] object.
///
/// Since tantivy stores it as a single `i64` nanosecond timestamp, it is serialized and
/// deserialized as one.
fn serialize_datetime<S: Serializer>(
  dt: &tv::DateTime,
  serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
  dt.into_timestamp_nanos().serialize(serializer)
}

/// Deserializes a [`tv::DateTime`] object.
///
/// Since tantivy stores it as a single `i64` nanosecond timestamp, it is serialized and
/// deserialized as one.
fn deserialize_datetime<'de, D>(deserializer: D) -> std::result::Result<tv::DateTime, D::Error>
where
  D: Deserializer<'de>,
{
  i64::deserialize(deserializer).map(tv::DateTime::from_timestamp_nanos)
}

fn deserialize_json_object_as_i64<'de, D>(
  deserializer: D,
) -> std::result::Result<Vec<(String, Value)>, D::Error>
where
  D: Deserializer<'de>,
{
  let raw_object = Vec::deserialize(deserializer)?;
  let converted_object = raw_object
    .into_iter()
    .map(|(key, value)| {
      let converted_value = match value {
        serde_json::Value::Number(num) => {
          if let Some(i) = num.as_i64() {
            Value::I64(i)
          } else {
            Value::F64(num.as_f64().unwrap())
          }
        }
        serde_json::Value::Object(obj) => Value::Object(deserialize_json_object_as_i64_inner(obj)),
        _ => Value::from(value),
      };
      (key, converted_value)
    })
    .collect();

  Ok(converted_object)
}

fn deserialize_json_object_as_i64_inner(
  raw_object: serde_json::Map<String, serde_json::Value>,
) -> Vec<(String, Value)> {
  raw_object
    .into_iter()
    .map(|(key, value)| {
      let converted_value = match value {
        serde_json::Value::Number(num) => {
          if let Some(i) = num.as_i64() {
            Value::I64(i)
          } else {
            Value::F64(num.as_f64().unwrap())
          }
        }
        serde_json::Value::Object(obj) => Value::Object(deserialize_json_object_as_i64_inner(obj)),
        _ => Value::from(value),
      };
      (key, converted_value)
    })
    .collect()
}

/// An equivalent type to [`tantivy::schema::Value`], but unlike the tantivy crate's serialization
/// implementation, it uses tagging in its serialization and deserialization to differentiate
/// between different integer types.
///
/// [`BorrowedSerdeValue`] is often used for the serialization path, as owning the data is not
/// necessary for serialization.
#[derive(Deserialize, Serialize)]
enum SerdeValue {
  /// Null
  Null,
  /// The str type is used for any text information.
  Str(String),
  /// Pre-tokenized str type,
  PreTokStr(tv::tokenizer::PreTokenizedString),
  /// Unsigned 64-bits Integer `u64`
  U64(u64),
  /// Signed 64-bits Integer `i64`
  I64(i64),
  /// 64-bits Float `f64`
  F64(f64),
  /// Bool value
  Bool(bool),
  #[serde(
    deserialize_with = "deserialize_datetime",
    serialize_with = "serialize_datetime"
  )]
  /// Date/time with microseconds precision
  Date(tv::DateTime),
  /// Facet
  Facet(tv::schema::Facet),
  /// Arbitrarily sized byte array
  Bytes(Vec<u8>),
  /// Array
  Array(Vec<Value>),
  /// Object value.
  #[serde(deserialize_with = "deserialize_json_object_as_i64")]
  Object(Vec<(String, Value)>),
  /// IpV6 Address. Internally there is no IpV4, it needs to be converted to `Ipv6Addr`.
  IpAddr(Ipv6Addr),
}

impl From<SerdeValue> for Value {
  fn from(value: SerdeValue) -> Self {
    match value {
      SerdeValue::Null => Self::Null,
      SerdeValue::Str(v) => Self::Str(v),
      SerdeValue::PreTokStr(v) => Self::PreTokStr(v),
      SerdeValue::U64(v) => Self::U64(v),
      SerdeValue::I64(v) => Self::I64(v),
      SerdeValue::F64(v) => Self::F64(v),
      SerdeValue::Date(v) => Self::Date(v),
      SerdeValue::Facet(v) => Self::Facet(v),
      SerdeValue::Bytes(v) => Self::Bytes(v),
      SerdeValue::Array(v) => Self::Array(v),
      SerdeValue::Object(v) => Self::Object(v),
      SerdeValue::Bool(v) => Self::Bool(v),
      SerdeValue::IpAddr(v) => Self::IpAddr(v),
    }
  }
}

impl From<Value> for SerdeValue {
  fn from(value: Value) -> Self {
    match value {
      Value::Null => Self::Null,
      Value::Str(v) => Self::Str(v),
      Value::PreTokStr(v) => Self::PreTokStr(v),
      Value::U64(v) => Self::U64(v),
      Value::I64(v) => Self::I64(v),
      Value::F64(v) => Self::F64(v),
      Value::Date(v) => Self::Date(v),
      Value::Facet(v) => Self::Facet(v),
      Value::Bytes(v) => Self::Bytes(v),
      Value::Array(v) => Self::Array(v),
      Value::Object(v) => Self::Object(v),
      Value::Bool(v) => Self::Bool(v),
      Value::IpAddr(v) => Self::IpAddr(v),
    }
  }
}

/// A non-owning version of [`SerdeValue`]. This is used in serialization to avoid unnecessary
/// cloning.
#[derive(Serialize)]
enum BorrowedSerdeValue<'a> {
  /// Null
  Null,
  /// The str type is used for any text information.
  Str(&'a str),
  /// Pre-tokenized str type,
  PreTokStr(&'a tv::tokenizer::PreTokenizedString),
  /// Unsigned 64-bits Integer `u64`
  U64(&'a u64),
  /// Signed 64-bits Integer `i64`
  I64(&'a i64),
  /// 64-bits Float `f64`
  F64(&'a f64),
  /// Bool value
  Bool(&'a bool),
  #[serde(serialize_with = "serialize_datetime")]
  /// Date/time with microseconds precision
  Date(&'a tv::DateTime),
  /// Facet
  Facet(&'a tv::schema::Facet),
  /// Arbitrarily sized byte array
  Bytes(&'a [u8]),
  /// Array
  Array(&'a Vec<Value>),
  /// Json object value.
  Object(&'a Vec<(String, Value)>),
  /// IpV6 Address. Internally there is no IpV4, it needs to be converted to `Ipv6Addr`.
  IpAddr(&'a Ipv6Addr),
}

impl<'a> From<&'a Value> for BorrowedSerdeValue<'a> {
  fn from(value: &'a Value) -> Self {
    match value {
      Value::Null => Self::Null,
      Value::Str(v) => Self::Str(v),
      Value::PreTokStr(v) => Self::PreTokStr(v),
      Value::U64(v) => Self::U64(v),
      Value::I64(v) => Self::I64(v),
      Value::F64(v) => Self::F64(v),
      Value::Date(v) => Self::Date(v),
      Value::Facet(v) => Self::Facet(v),
      Value::Bytes(v) => Self::Bytes(v),
      Value::Array(v) => Self::Array(v),
      Value::Object(v) => Self::Object(v),
      Value::Bool(v) => Self::Bool(v),
      Value::IpAddr(v) => Self::IpAddr(v),
    }
  }
}

/// Tantivy's Document is the object that can be indexed and then searched for.
///
/// Documents are fundamentally a collection of unordered tuples
/// (field_name, value). In this list, one field may appear more than once.
///
/// Example:
///     const doc = new Document();
///     doc.addText("title", "The Old Man and the Sea");
///     doc.addText("body", "He was an old man who fished alone in a " +
///                         "skiff in the Gulf Stream and he had gone " +
///                         "eighty-four days now without taking a fish.");
///     console.log(doc.toString());
///
/// For simplicity, it is also possible to build a `Document` by passing the field
/// values directly as constructor arguments.
///
/// Example:
///     const doc = Document.fromObject({
///         title: "The Old Man and the Sea",
///         body: "..."
///     });
///
/// For numeric fields, the `Document` constructor does not have any
/// information about the type and will try to guess the type.
/// Therefore, it is recommended to use the `Document.fromObject()`,
/// or `Document.add*()` functions to provide
/// explicit type information.
#[napi]
#[derive(Clone, Default, PartialEq)]
pub struct Document {
  pub(crate) field_values: BTreeMap<String, Vec<Value>>,
}

impl fmt::Debug for Document {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let doc_str = self
      .field_values
      .iter()
      .map(|(field_name, field_values)| {
        let values_str: String = field_values
          .iter()
          .map(value_to_string)
          .collect::<Vec<_>>()
          .join(",")
          .chars()
          .take(10)
          .collect();
        format!("{field_name}=[{values_str}]")
      })
      .collect::<Vec<_>>()
      .join(",");
    write!(f, "Document({doc_str})")
  }
}

impl Serialize for Document {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.field_values.len()))?;
    for (k, v) in &self.field_values {
      let ser_v: Vec<_> = v.iter().map(BorrowedSerdeValue::from).collect();
      map.serialize_entry(&k, &ser_v)?;
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for Document {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    BTreeMap::<String, Vec<SerdeValue>>::deserialize(deserializer).map(|field_map| Document {
      field_values: field_map
        .into_iter()
        .map(|(k, v)| {
          let v: Vec<_> = v.into_iter().map(Value::from).collect();
          (k, v)
        })
        .collect(),
    })
  }
}

#[napi]
impl Document {
  /// Creates a new document.
  #[napi(constructor)]
  pub fn new() -> Self {
    Document::default()
  }

  /// Extend the document with field values from a JavaScript object.
  #[napi]
  pub fn extend(&mut self, env: Env, js_obj: Object, schema: Option<&Schema>) -> Result<()> {
    Document::extract_js_values_from_object(env, &js_obj, schema, &mut self.field_values)
  }

  /// Create a document from a JavaScript object.
  #[napi(factory)]
  pub fn from_dict(env: Env, js_obj: Object, schema: Option<&Schema>) -> Result<Document> {
    let mut field_values: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    Document::extract_js_values_from_object(env, &js_obj, schema, &mut field_values)?;
    Ok(Document { field_values })
  }

  /// Returns a JavaScript object with the different field values.
  ///
  /// In tantivy, `Document` can hold multiple values for a single field.
  ///
  /// For this reason, the object will associate a list of values for every field.
  #[napi]
  pub fn to_dict(&self, env: Env) -> Result<Object> {
    let mut obj = Object::new(&env)?;
    for (key, values) in &self.field_values {
      let mut js_values = env.create_array(values.len() as u32)?;
      for (i, v) in values.iter().enumerate() {
        let js_value = value_to_js(env, v)?;
        js_values.set_element(i as u32, js_value)?;
      }
      obj.set_named_property(key, js_values)?;
    }
    Ok(obj)
  }

  /// Add a text value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the text.
  /// @param text - The text that will be added to the document.
  #[napi]
  pub fn add_text(&mut self, field_name: String, text: String) {
    self.add_value(field_name, text);
  }

  /// Add an unsigned integer value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the unsigned integer.
  /// @param value - The integer that will be added to the document.
  #[napi]
  pub fn add_unsigned(&mut self, field_name: String, value: u32) {
    self.add_value(field_name, value as u64);
  }

  /// Add a signed integer value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the integer.
  /// @param value - The integer that will be added to the document.
  #[napi]
  pub fn add_integer(&mut self, field_name: String, value: i64) {
    self.add_value(field_name, value);
  }

  /// Add a float value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the value.
  /// @param value - The float that will be added to the document.
  #[napi]
  pub fn add_float(&mut self, field_name: String, value: f64) {
    self.add_value(field_name, value);
  }

  /// Add a boolean value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the value.
  /// @param value - The boolean that will be added to the document.
  #[napi]
  pub fn add_boolean(&mut self, field_name: String, value: bool) {
    self.add_value(field_name, value);
  }

  /// Add a date value to the document.
  ///
  /// @param fieldName - The field name for which we are adding the date.
  /// @param timestampMillis - The date timestamp in milliseconds (JavaScript time) that will be added to the document.
  #[napi]
  pub fn add_date(&mut self, field_name: String, timestamp_millis: i64) {
    self.add_value(
      field_name,
      tv::DateTime::from_timestamp_secs(timestamp_millis / 1000),
    );
  }

  /// Add a facet value to the document.
  /// @param fieldName - The field name for which we are adding the facet.
  /// @param facet - The Facet that will be added to the document.
  #[napi]
  pub fn add_facet(&mut self, field_name: String, facet: &Facet) {
    self.add_value(field_name, facet.inner.clone());
  }

  /// Add a bytes value to the document.
  ///
  /// @param fieldName - The field for which we are adding the bytes.
  /// @param bytes - The bytes (as Buffer or Uint8Array) that will be added to the document.
  #[napi]
  pub fn add_bytes(&mut self, field_name: String, bytes: &[u8]) {
    self.add_value(field_name, bytes.to_vec());
  }

  /// Add a JSON value to the document.
  ///
  /// @param fieldName - The field for which we are adding the JSON.
  /// @param value - The JSON object that will be added to the document.
  ///
  /// @throws Raises an error if the JSON is invalid.
  #[napi]
  pub fn add_json(&mut self, field_name: String, value: Object) -> Result<()> {
    let json_value = js_object_to_json_value(value)?;
    // Use Value::from(serde_json::Value) directly - no need for manual conversion!
    let tantivy_value = Value::from(json_value);
    self.add_value(field_name, tantivy_value);
    Ok(())
  }

  /// Add an IP address value to the document.
  ///
  /// @param fieldName - The field for which we are adding the IP address.
  /// @param value - The IP address string that will be added to the document.
  ///
  /// @throws Raises an error if the IP address is invalid.
  #[napi]
  pub fn add_ip_addr(&mut self, field_name: String, value: String) -> Result<()> {
    let ip_addr = IpAddr::from_str(&value)
      .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid IP address: {}", e)))?;
    match ip_addr {
      IpAddr::V4(addr) => self.add_value(field_name, addr.to_ipv6_mapped()),
      IpAddr::V6(addr) => self.add_value(field_name, addr),
    }
    Ok(())
  }

  /// Returns the number of added fields that have been added to the document
  #[napi(getter)]
  pub fn num_fields(&self) -> u32 {
    self.field_values.len() as u32
  }

  /// True if the document is empty, False otherwise.
  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.field_values.is_empty()
  }

  /// Get the first value associated with the given field.
  ///
  /// @param fieldName - The field for which we would like to get the value.
  ///
  /// @returns The value if one is found, otherwise undefined.
  /// The type of the value depends on the field.
  #[napi]
  pub fn get_first(&self, env: Env, field_name: String) -> Result<Unknown> {
    if let Some(value) = self.iter_values_for_field(&field_name).next() {
      value_to_js(env, value)
    } else {
      env.to_js_value(&()) // Returns undefined
    }
  }

  /// Get all values associated with the given field.
  ///
  /// @param fieldName - The field for which we would like to get the values.
  ///
  /// @returns An array of values.
  /// The type of the value depends on the field.
  #[napi]
  pub fn get_all(&self, env: Env, field_name: String) -> Result<Unknown> {
    let values: Vec<serde_json::Value> = self
      .iter_values_for_field(&field_name)
      .map(|value| value_to_serde_json(value))
      .collect();
    env.to_js_value(&values)
  }

  /// Convert the document to a string representation
  #[napi]
  pub fn to_string(&self) -> String {
    format!("{self:?}")
  }
}

impl Document {
  fn add_value<T>(&mut self, field_name: String, value: T)
  where
    Value: From<T>,
  {
    self
      .field_values
      .entry(field_name)
      .or_default()
      .push(Value::from(value));
  }

  fn extract_js_values_from_object(
    _env: Env,
    js_object: &Object,
    schema: Option<&Schema>,
    out_field_values: &mut BTreeMap<String, Vec<Value>>,
  ) -> Result<()> {
    let keys = js_object.get_property_names()?;
    for i in 0..keys.get_array_length()? {
      let key: String = keys.get_element(i)?;
      let js_value: Unknown = js_object.get_named_property(&key)?;

      let field_type = if let Some(schema) = schema {
        match schema.inner.get_field(&key) {
          Ok(field) => {
            let field_entry = schema.inner.get_field_entry(field);
            Some(field_entry.field_type().clone())
          }
          Err(_) => continue, // Skip fields not in schema
        }
      } else {
        None
      };

      let value_list = if let Some(ref field_type) = field_type {
        extract_value_single_or_list_for_type(&js_value, field_type, &key)?
      } else {
        extract_value_single_or_list(&js_value)?
      };

      out_field_values.insert(key, value_list);
    }
    Ok(())
  }

  pub fn iter_values_for_field<'a>(&'a self, field: &str) -> impl Iterator<Item = &'a Value> + 'a {
    self
      .field_values
      .get(field)
      .into_iter()
      .flat_map(|values| values.iter())
  }
}
