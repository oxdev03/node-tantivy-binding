#![allow(clippy::new_ret_no_self)]
#![allow(clippy::wrong_self_convention)]

use chrono::{DateTime, Utc};
use napi::{
  Env, Error, JsBoolean, JsBuffer, JsNumber, JsObject, JsString, JsUnknown, Result, Status,
  ValueType,
};
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
  pub fn extend(&mut self, dict: JsObject, schema: Option<&Schema>) -> Result<()> {
    Document::extract_js_values_from_dict(dict, schema, &mut self.field_values)
  }

  /// Create a document from a dictionary with optional schema.
  ///
  /// @param dict - Object containing field names and values
  /// @param schema - Optional schema for type validation
  #[napi(factory)]
  pub fn from_dict(dict: JsObject, schema: Option<&Schema>) -> Result<Document> {
    let mut field_values: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    Document::extract_js_values_from_dict(dict, schema, &mut field_values)?;
    Ok(Document { field_values })
  }

  /// Returns a dictionary with the different
  /// field values.
  #[napi]
  pub fn to_dict(&self, env: Env) -> Result<JsObject> {
    let mut dict = env.create_object()?;
    for (key, values) in &self.field_values {
      let mut arr = env.create_array_with_length(values.len())?;
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
  pub fn add_bytes(&mut self, field_name: String, bytes: JsBuffer) -> Result<()> {
    self.add_value(field_name, bytes.into_value()?.to_vec());
    Ok(())
  }

  /// Add a JSON value to the document.
  ///
  /// @param field_name - The field for which we are adding the JSON.
  /// @param value - The JSON object as a string or object.
  #[napi]
  pub fn add_json(&mut self, field_name: String, value: JsObject) -> Result<()> {
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
  pub fn get_first(&self, env: Env, field_name: String) -> Result<JsUnknown> {
    if let Some(value) = self.iter_values_for_field(&field_name).next() {
      value_to_js(env, value)
    } else {
      Ok(env.get_null()?.into_unknown())
    }
  }

  /// Get the all values associated with the given field.
  ///
  /// @param field_name - The field for which we would like to get the values.
  /// @returns A list of values.
  #[napi]
  pub fn get_all(&self, env: Env, field_name: String) -> Result<JsObject> {
    let values: Vec<JsUnknown> = self
      .iter_values_for_field(&field_name)
      .map(|value| value_to_js(env, value))
      .collect::<Result<Vec<_>>>()?;
    let mut arr = env.create_array_with_length(values.len())?;
    for (i, v) in values.iter().enumerate() {
      arr.set_element(i as u32, v)?;
    }
    Ok(arr)
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
    dict: JsObject,
    schema: Option<&Schema>,
    out_field_values: &mut BTreeMap<String, Vec<Value>>,
  ) -> Result<()> {
    let keys = dict.get_property_names()?;
    for i in 0..keys.get_array_length()? {
      let key_js: JsString = keys.get_element(i)?;
      let key: String = key_js.into_utf8()?.as_str()?.to_string();
      let value_js: JsUnknown = dict.get_property(key_js)?;

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

fn js_object_to_json_value(obj: JsObject) -> Result<serde_json::Value> {
  let keys = obj.get_property_names()?;
  let mut map = serde_json::Map::new();

  for i in 0..keys.get_array_length()? {
    let key_js: JsString = keys.get_element(i)?;
    let key: String = key_js.into_utf8()?.as_str()?.to_string();
    let value_js: JsUnknown = obj.get_property(key_js)?;

    let json_value = match value_js.get_type()? {
      ValueType::String => {
        let s: JsString = unsafe { value_js.cast() };
        serde_json::Value::String(s.into_utf8()?.as_str()?.to_string())
      }
      ValueType::Number => {
        let n: JsNumber = unsafe { value_js.cast() };
        serde_json::Value::Number(
          serde_json::Number::from_f64(n.get_double()?)
            .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid number".to_string()))?,
        )
      }
      ValueType::Boolean => {
        let b: JsBoolean = unsafe { value_js.cast() };
        serde_json::Value::Bool(b.get_value()?)
      }
      ValueType::Object => {
        let inner_obj: JsObject = unsafe { value_js.cast() };
        if value_js.is_array()? {
          let len = inner_obj.get_array_length()?;
          let mut arr = Vec::new();
          for j in 0..len {
            let elem: JsUnknown = inner_obj.get_element(j)?;
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

fn js_unknown_to_json_value(value: JsUnknown) -> Result<serde_json::Value> {
  match value.get_type()? {
    ValueType::String => {
      let s: JsString = unsafe { value.cast() };
      Ok(serde_json::Value::String(
        s.into_utf8()?.as_str()?.to_string(),
      ))
    }
    ValueType::Number => {
      let n: JsNumber = unsafe { value.cast() };
      Ok(serde_json::Value::Number(
        serde_json::Number::from_f64(n.get_double()?)
          .ok_or_else(|| Error::new(Status::InvalidArg, "Invalid number".to_string()))?,
      ))
    }
    ValueType::Boolean => {
      let b: JsBoolean = unsafe { value.cast() };
      Ok(serde_json::Value::Bool(b.get_value()?))
    }
    ValueType::Object => {
      let obj: JsObject = unsafe { value.cast() };
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

fn extract_value(any: &JsUnknown) -> Result<Value> {
  match any.get_type()? {
    ValueType::String => {
      let s: JsString = unsafe { any.cast() };
      Ok(Value::Str(s.into_utf8()?.as_str()?.to_string()))
    }
    ValueType::Number => {
      let n: JsNumber = unsafe { any.cast() };
      Ok(Value::F64(n.get_double()?))
    }
    ValueType::Boolean => {
      let b: JsBoolean = unsafe { any.cast() };
      Ok(Value::Bool(b.get_value()?))
    }
    ValueType::Object => {
      let obj: JsObject = unsafe { any.cast() };
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
  any: &JsUnknown,
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
          let s: JsString = unsafe { any.cast() };
          Value::Str(s.into_utf8()?.as_str()?.to_string())
        }
        _ => {
          // Try coercing to string as fallback
          let s: JsString = unsafe { any.cast() };
          Value::Str(s.into_utf8()?.as_str()?.to_string())
        }
      }
    }
    tv::schema::Type::U64 => {
      let n: JsNumber = unsafe { any.cast() };
      Value::U64(n.get_uint32()? as u64) // Note: potential precision loss
    }
    tv::schema::Type::I64 => {
      let n: JsNumber = unsafe { any.cast() };
      Value::I64(n.get_int64()?)
    }
    tv::schema::Type::F64 => {
      let n: JsNumber = unsafe { any.cast() };
      Value::F64(n.get_double()?)
    }
    tv::schema::Type::Bool => {
      let b: JsBoolean = unsafe { any.cast() };
      Value::Bool(b.get_value()?)
    }
    tv::schema::Type::Date => {
      let s: JsString = unsafe { any.cast() };
      let s = s.into_utf8()?.as_str()?.to_string();
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
      let s: JsString = unsafe { any.cast() };
      let s = s.into_utf8()?.as_str()?.to_string();
      Value::Facet(tv::schema::Facet::from(s.as_str()))
    }
    tv::schema::Type::Bytes => {
      let buf: JsBuffer = unsafe { any.cast() };
      Value::Bytes(buf.into_value()?.to_vec())
    }
    tv::schema::Type::Json => {
      let obj: JsObject = unsafe { any.cast() };
      let json_value = js_object_to_json_value(obj)?;
      json_to_tantivy_value(json_value)?
    }
    tv::schema::Type::IpAddr => {
      let s: JsString = unsafe { any.cast() };
      let s = s.into_utf8()?.as_str()?.to_string();
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

fn extract_value_single_or_list(any: &JsUnknown) -> Result<Vec<Value>> {
  if any.is_array()? {
    let obj: JsObject = unsafe { any.cast() };
    let len = obj.get_array_length()?;
    let mut values = Vec::with_capacity(len as usize);
    for i in 0..len {
      let elem: JsUnknown = obj.get_element(i)?;
      values.push(extract_value(&elem)?);
    }
    Ok(values)
  } else {
    Ok(vec![extract_value(any)?])
  }
}

fn extract_value_single_or_list_for_type(
  any: &JsUnknown,
  field_type: &tv::schema::FieldType,
  field_name: &str,
) -> Result<Vec<Value>> {
  if any.is_array()? {
    let obj: JsObject = unsafe { any.cast() };
    let len = obj.get_array_length()?;
    let mut values = Vec::with_capacity(len as usize);
    for i in 0..len {
      let elem: JsUnknown = obj.get_element(i)?;
      values.push(extract_value_for_type(&elem, field_type, field_name)?);
    }
    Ok(values)
  } else {
    Ok(vec![extract_value_for_type(any, field_type, field_name)?])
  }
}

fn value_to_js(env: Env, value: &Value) -> Result<JsUnknown> {
  Ok(match value {
    Value::Str(text) => env.create_string(text)?.into_unknown(),
    Value::U64(num) => env.create_double(*num as f64)?.into_unknown(),
    Value::I64(num) => env.create_double(*num as f64)?.into_unknown(),
    Value::F64(num) => env.create_double(*num)?.into_unknown(),
    Value::Bytes(b) => env.create_buffer_with_data(b.clone())?.into_unknown(),
    Value::Date(d) => {
      // Format the date using time's Iso8601 formatter
      let formatted = d
        .into_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|_| Error::new(Status::GenericFailure, "Failed to format date".to_string()))?;
      env.create_string(&formatted)?.into_unknown()
    }
    Value::Facet(f) => env.create_string(&f.to_string())?.into_unknown(),
    Value::Object(obj) => {
      let mut js_obj = env.create_object()?;
      for (k, v) in obj.iter() {
        js_obj.set_named_property(k, value_to_js(env, v)?)?;
      }
      js_obj.into_unknown()
    }
    Value::Array(arr) => {
      let mut js_arr = env.create_array_with_length(arr.len())?;
      for (i, v) in arr.iter().enumerate() {
        js_arr.set_element(i as u32, value_to_js(env, v)?)?;
      }
      js_arr.into_unknown()
    }
    Value::Bool(b) => env.get_boolean(*b)?.into_unknown(),
    Value::IpAddr(i) => env.create_string(&i.to_string())?.into_unknown(),
    _ => env.get_null()?.into_unknown(), // PreTokStr and Array not handled directly
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
