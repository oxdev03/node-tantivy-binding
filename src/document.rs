#![allow(clippy::new_ret_no_self)]
#![allow(clippy::wrong_self_convention)]

use chrono::{DateTime, Utc};
use napi::bindgen_prelude::*;
use napi::{Env, Error, Result, Status};
use napi_derive::napi;
use std::{collections::BTreeMap, fmt, net::IpAddr, str::FromStr};
use tantivy::{self as tv, schema::document::OwnedValue as Value, time};

use crate::schema::Schema;

/// Tantivy's Document is the object that can be indexed and then searched for.
///
/// Documents are fundamentally a collection of unordered tuples
/// (field_name, value). In this list, one field may appear more than once.
///
/// Example:
/// ```javascript
/// const doc = new Document();
/// doc.addText("title", "The Old Man and the Sea");
/// doc.addText("body", "He was an old man who fished alone in a skiff...");
/// ```
#[napi]
#[derive(Clone, Default, PartialEq)]
pub struct Document {
  pub(crate) field_values: BTreeMap<String, Vec<Value>>,
}

impl fmt::Debug for Document {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let doc_str: String = self
      .field_values
      .iter()
      .map(|(field_name, field_values)| {
        let values_str: String = field_values
          .iter()
          .map(value_to_string)
          .collect::<Vec<String>>()
          .join(",")
          .chars()
          .take(10)
          .collect();
        format!("{field_name}=[{values_str}]")
      })
      .collect::<Vec<String>>()
      .join(",");
    write!(f, "Document({doc_str})")
  }
}

#[napi]
impl Document {
  /// Creates a new document.
  #[napi(constructor)]
  pub fn new() -> Self {
    Document::default()
  }

  /// Extend the document with values from a dictionary.
  ///
  /// @param dict - Object containing field names and values to add
  /// @param schema - Optional schema for type validation
  #[napi]
  pub fn extend(&mut self, dict: Object, schema: Option<&Schema>) -> Result<()> {
    Document::extract_js_values_from_dict(dict, schema, &mut self.field_values)
  }

  /// Create a document from a dictionary with optional schema.
  ///
  /// @param dict - Object containing field names and values
  /// @param schema - Optional schema for type validation
  #[napi(factory)]
  pub fn from_dict(dict: Object, schema: Option<&Schema>) -> Result<Document> {
    let mut field_values: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    Document::extract_js_values_from_dict(dict, schema, &mut field_values)?;
    Ok(Document { field_values })
  }

  /// Returns a dictionary with the different
  /// field values.
  #[napi]
  pub fn to_dict(&self, env: Env) -> Result<Object> {
    let mut dict = Object::new(&env)?;
    for (key, values) in &self.field_values {
      let mut arr = env.create_array(values.len() as u32)?;
      for (i, value) in values.iter().enumerate() {
        arr.set_element(i as u32, value_to_js(env, value)?)?;
      }
      dict.set_named_property(key, arr)?;
    }
    Ok(dict)
  }

  /// Add a text value to the document.
  ///
  /// @param field_name - The field name for which we are adding the text.
  /// @param text - The text that will be added to the document.
  #[napi]
  pub fn add_text(&mut self, field_name: String, text: String) {
    self.add_value(field_name, text);
  }

  /// Add an unsigned integer value to the document.
  ///
  /// @param field_name - The field name for which we are adding the unsigned integer.
  /// @param value - The integer that will be added to the document.
  #[napi]
  pub fn add_unsigned(&mut self, field_name: String, value: f64) {
    self.add_value(field_name, value as u64);
  }

  /// Add a signed integer value to the document.
  ///
  /// @param field_name - The field name for which we are adding the integer.
  /// @param value - The integer that will be added to the document.
  #[napi]
  pub fn add_integer(&mut self, field_name: String, value: f64) {
    self.add_value(field_name, value as i64);
  }

  /// Add a float value to the document.
  ///
  /// @param field_name - The field name for which we are adding the value.
  /// @param value - The float that will be added to the document.
  #[napi]
  pub fn add_float(&mut self, field_name: String, value: f64) {
    self.add_value(field_name, value);
  }

  /// Add a boolean value to the document.
  ///
  /// @param field_name - The field name for which we are adding the value.
  /// @param value - The boolean that will be added to the document.
  #[napi]
  pub fn add_boolean(&mut self, field_name: String, value: bool) {
    self.add_value(field_name, value);
  }

  /// Add a date value to the document.
  ///
  /// @param field_name - The field name for which we are adding the date.
  /// @param value - The date as a string in RFC 3339 format or a Unix timestamp (in seconds).
  #[napi]
  pub fn add_date(&mut self, field_name: String, value: String) -> Result<()> {
    let date = if let Ok(dt) = DateTime::parse_from_rfc3339(&value) {
      dt.with_timezone(&Utc)
    } else if let Ok(timestamp) = value.parse::<i64>() {
      DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid timestamp".to_string()))?
    } else {
      return Err(Error::new(
        Status::InvalidArg,
        "Date must be an ISO 8601 string or Unix timestamp".to_string(),
      ));
    };
    let tantivy_date = tv::DateTime::from_timestamp_secs(date.timestamp());
    self.add_value(field_name, tantivy_date);
    Ok(())
  }

  /// Add a facet value to the document.
  /// @param field_name - The field name for which we are adding the facet.
  /// @param value - The Facet that will be added to the document.
  #[napi]
  pub fn add_facet(&mut self, field_name: String, facet_path: String) -> Result<()> {
    let facet = tv::schema::Facet::from(facet_path.as_str());
    self.add_value(field_name, facet);
    Ok(())
  }

  /// Add a bytes value to the document.
  ///
  /// @param field_name - The field for which we are adding the bytes.
  /// @param value - The bytes that will be added to the document.
  #[napi]
  pub fn add_bytes(&mut self, field_name: String, bytes: Buffer) -> Result<()> {
    self.add_value(field_name, bytes.to_vec());
    Ok(())
  }

  /// Add a JSON value to the document.
  ///
  /// @param field_name - The field for which we are adding the JSON.
  /// @param value - The JSON object as a string or object.
  #[napi]
  pub fn add_json(&mut self, field_name: String, value: Object) -> Result<()> {
    let json_value = js_object_to_json_value(value)?;
    let tantivy_value = json_to_tantivy_value(json_value)?;
    self.add_value(field_name, tantivy_value);
    Ok(())
  }

  /// Add an IP address value to the document.
  ///
  /// @param field_name - The field for which we are adding the IP address.
  /// @param value - The IP address object that will be added to the document.
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
  /// @param field_name - The field for which we would like to get the value.
  /// @returns The value if one is found, otherwise null.
  #[napi]
  pub fn get_first(&self, env: Env, field_name: String) -> Result<Unknown> {
    if let Some(value) = self.iter_values_for_field(&field_name).next() {
      value_to_js(env, value)
    } else {
      env.to_js_value(&()) // Returns undefined
    }
  }

  /// Get the all values associated with the given field.
  ///
  /// @param field_name - The field for which we would like to get the values.
  /// @returns A list of values.
  #[napi]
  pub fn get_all(&self, env: Env, field_name: String) -> Result<Unknown> {
    let values: Vec<serde_json::Value> = self
      .iter_values_for_field(&field_name)
      .map(|value| value_to_serde_json(value))
      .collect();
    env.to_js_value(&values)
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

  fn extract_js_values_from_dict(
    dict: Object,
    schema: Option<&Schema>,
    out_field_values: &mut BTreeMap<String, Vec<Value>>,
  ) -> Result<()> {
    let keys = dict.get_property_names()?;
    for i in 0..keys.get_array_length()? {
      let key_js = keys.get_element::<String>(i)?;
      let key: String = key_js;
      let value_js: Unknown = dict.get_named_property(&key)?;

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
        extract_value_single_or_list_for_type(&value_js, field_type, &key)?
      } else {
        extract_value_single_or_list(&value_js)?
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

fn to_napi_error(e: impl std::error::Error) -> Error {
  Error::new(Status::GenericFailure, format!("{}", e))
}

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
      ValueType::Boolean => match value_js.coerce_to_bool() {
        Ok(b) => serde_json::Value::Bool(b),
        Err(_) => serde_json::Value::Bool(false),
      },
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
      let bool_value = value.coerce_to_bool()?;
      Ok(serde_json::Value::Bool(bool_value))
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

fn json_to_tantivy_value(json: serde_json::Value) -> Result<Value> {
  match json {
    serde_json::Value::Null => Err(Error::new(
      Status::InvalidArg,
      "Null is not supported".to_string(),
    )),
    serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
    serde_json::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        Ok(Value::I64(i))
      } else if let Some(u) = n.as_u64() {
        Ok(Value::U64(u))
      } else if let Some(f) = n.as_f64() {
        Ok(Value::F64(f))
      } else {
        Err(Error::new(
          Status::InvalidArg,
          "Invalid number type".to_string(),
        ))
      }
    }
    serde_json::Value::String(s) => Ok(Value::Str(s)),
    serde_json::Value::Array(arr) => {
      let mut tantivy_array = Vec::new();
      for item in arr {
        tantivy_array.push(json_to_tantivy_value(item)?);
      }
      Ok(Value::Array(tantivy_array))
    }
    serde_json::Value::Object(m) => {
      let mut obj = Vec::new();
      for (k, v) in m {
        obj.push((k, json_to_tantivy_value(v)?));
      }
      Ok(Value::Object(obj))
    }
  }
}

fn extract_value(any: &Unknown) -> Result<Value> {
  match any.get_type()? {
    ValueType::String => {
      let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
      Ok(Value::Str(s))
    }
    ValueType::Number => {
      let n: f64 = any.coerce_to_number()?.get_double()?;
      Ok(Value::F64(n))
    }
    ValueType::Boolean => {
      let b: bool = any.coerce_to_bool()?;
      Ok(Value::Bool(b))
    }
    ValueType::Object => {
      let obj: Object = unsafe { any.cast()? };
      let json_value = js_object_to_json_value(obj)?;
      json_to_tantivy_value(json_value)
    }
    _ => Err(Error::new(
      Status::InvalidArg,
      "Unsupported value type".to_string(),
    )),
  }
}

fn extract_value_for_type(
  any: &Unknown,
  tv_type: &tv::schema::FieldType,
  field_name: &str,
) -> Result<Value> {
  let _type_error = |expected: &str| -> Error {
    Error::new(
      Status::InvalidArg,
      format!(
        "Expected {} type for field {}, got another type",
        expected, field_name
      ),
    )
  };

  // Convert based on the target type using direct casting and type checking
  let value = match tv_type.value_type() {
    tv::schema::Type::Str => {
      match any.get_type()? {
        ValueType::String => {
          let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
          Value::Str(s)
        }
        _ => {
          // Try coercing to string as fallback
          let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
          Value::Str(s)
        }
      }
    }
    tv::schema::Type::U64 => {
      let n: f64 = any.coerce_to_number()?.get_double()?;
      Value::U64(n as u64) // Note: potential precision loss
    }
    tv::schema::Type::I64 => {
      let n: f64 = any.coerce_to_number()?.get_double()?;
      Value::I64(n as i64)
    }
    tv::schema::Type::F64 => {
      let n: f64 = any.coerce_to_number()?.get_double()?;
      Value::F64(n)
    }
    tv::schema::Type::Bool => {
      let b: bool = any.coerce_to_bool()?;
      Value::Bool(b)
    }
    tv::schema::Type::Date => {
      let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
      let date = if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        dt.with_timezone(&Utc)
      } else if let Ok(timestamp) = s.parse::<i64>() {
        DateTime::from_timestamp(timestamp, 0)
          .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid timestamp".to_string()))?
      } else {
        return Err(Error::new(
          Status::InvalidArg,
          "Date must be an ISO 8601 string or Unix timestamp".to_string(),
        ));
      };
      Value::Date(tv::DateTime::from_timestamp_secs(date.timestamp()))
    }
    tv::schema::Type::Facet => {
      let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
      Value::Facet(tv::schema::Facet::from(s.as_str()))
    }
    tv::schema::Type::Bytes => {
      let buf: Buffer = unsafe { any.cast()? };
      Value::Bytes(buf.to_vec())
    }
    tv::schema::Type::Json => {
      let obj: Object = unsafe { any.cast()? };
      let json_value = js_object_to_json_value(obj)?;
      json_to_tantivy_value(json_value)?
    }
    tv::schema::Type::IpAddr => {
      let s: String = any.coerce_to_string()?.into_utf8()?.as_str()?.to_string();
      let ip_addr = IpAddr::from_str(&s).map_err(to_napi_error)?;
      let ipv6_addr = match ip_addr {
        IpAddr::V4(addr) => addr.to_ipv6_mapped(),
        IpAddr::V6(addr) => addr,
      };
      Value::IpAddr(ipv6_addr)
    }
  };

  Ok(value)
}

fn extract_value_single_or_list(any: &Unknown) -> Result<Vec<Value>> {
  if any.is_array()? {
    let obj: Object = unsafe { any.cast()? };
    let len = obj.get_array_length()?;
    let mut values = Vec::with_capacity(len as usize);
    for i in 0..len {
      let elem: Unknown = obj.get_element(i)?;
      values.push(extract_value(&elem)?);
    }
    Ok(values)
  } else {
    Ok(vec![extract_value(any)?])
  }
}

fn extract_value_single_or_list_for_type(
  any: &Unknown,
  field_type: &tv::schema::FieldType,
  field_name: &str,
) -> Result<Vec<Value>> {
  if any.is_array()? {
    let obj: Object = unsafe { any.cast()? };
    let len = obj.get_array_length()?;
    let mut values = Vec::with_capacity(len as usize);
    for i in 0..len {
      let elem: Unknown = obj.get_element(i)?;
      values.push(extract_value_for_type(&elem, field_type, field_name)?);
    }
    Ok(values)
  } else {
    Ok(vec![extract_value_for_type(any, field_type, field_name)?])
  }
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
      let formatted = d
        .into_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap_or_else(|_| "".to_string());
      serde_json::Value::String(formatted)
    }
    Value::Facet(f) => serde_json::Value::String(f.to_string()),
    Value::Bytes(b) => serde_json::Value::Array(
      b.iter()
        .map(|&byte| serde_json::Value::Number(serde_json::Number::from(byte)))
        .collect(),
    ),
    Value::IpAddr(ip) => serde_json::Value::String(ip.to_string()),
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
    _ => serde_json::Value::Null, // For types we don't handle
  }
}

fn value_to_js(env: Env, value: &Value) -> Result<Unknown> {
  Ok(match value {
    Value::Str(text) => env.to_js_value(&text.as_str())?,
    Value::U64(num) => env.to_js_value(&(*num as f64))?,
    Value::I64(num) => env.to_js_value(&(*num as f64))?,
    Value::F64(num) => env.to_js_value(num)?,
    Value::Bytes(b) => {
      // For bytes, we need to create a buffer differently in v3
      env.to_js_value(&b.as_slice())?
    }
    Value::Date(d) => {
      // Format the date using time's Iso8601 formatter
      let formatted = d
        .into_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|_| Error::new(Status::GenericFailure, "Failed to format date".to_string()))?;
      env.to_js_value(&formatted)?
    }
    Value::Facet(f) => env.to_js_value(&f.to_string())?,
    Value::Object(obj) => {
      // For objects we need to convert manually since napi doesn't serialize BTreeMap directly
      let map: std::collections::HashMap<String, serde_json::Value> = obj
        .iter()
        .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
        .collect();
      env.to_js_value(&map)?
    }
    Value::Array(arr) => {
      let vec: Vec<serde_json::Value> = arr.iter().map(|v| value_to_serde_json(v)).collect();
      env.to_js_value(&vec)?
    }
    Value::Bool(b) => env.to_js_value(b)?,
    Value::IpAddr(i) => env.to_js_value(&i.to_string())?,
    _ => {
      env.to_js_value(&())? // () converts to undefined in v3
    } // PreTokStr and Array not handled directly
  })
}

fn value_to_string(value: &Value) -> String {
  match value {
    Value::Str(text) => text.clone(),
    Value::U64(num) => format!("{num}"),
    Value::I64(num) => format!("{num}"),
    Value::F64(num) => format!("{num}"),
    Value::Bytes(bytes) => format!("{bytes:?}"),
    Value::Date(d) => format!("{d:?}"),
    Value::Facet(facet) => facet.to_string(),
    Value::Object(json_object) => {
      serde_json::to_string(&json_object).unwrap_or_else(|_| "{}".to_string())
    }
    Value::Array(arr) => {
      let arr_strings: Vec<String> = arr.iter().map(value_to_string).collect();
      format!("[{}]", arr_strings.join(","))
    }
    Value::Bool(b) => format!("{b}"),
    Value::IpAddr(i) => format!("{}", *i),
    _ => "".to_string(),
  }
}
