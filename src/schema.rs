use napi_derive::napi;
use serde_json;
use tantivy as tv;
use tantivy::schema::Schema as TantivySchema;

/// Tantivy's FieldType
#[napi]
#[derive(PartialEq, Clone)]
pub enum FieldType {
  Str,
  U64,
  I64,
  F64,
  Bool,
  Date,
  Facet,
  Bytes,
  JsonObject,
  IpAddr,
}

impl From<FieldType> for tv::schema::Type {
  fn from(field_type: FieldType) -> tv::schema::Type {
    match field_type {
      FieldType::Str => tv::schema::Type::Str,
      FieldType::U64 => tv::schema::Type::U64,
      FieldType::I64 => tv::schema::Type::I64,
      FieldType::F64 => tv::schema::Type::F64,
      FieldType::Bool => tv::schema::Type::Bool,
      FieldType::Date => tv::schema::Type::Date,
      FieldType::Facet => tv::schema::Type::Facet,
      FieldType::Bytes => tv::schema::Type::Bytes,
      FieldType::JsonObject => tv::schema::Type::Json,
      FieldType::IpAddr => tv::schema::Type::IpAddr,
    }
  }
}

impl FieldType {
  pub fn from_tantivy_type(field_type: &tv::schema::Type) -> Self {
    match field_type {
      tv::schema::Type::Str => FieldType::Str,
      tv::schema::Type::U64 => FieldType::U64,
      tv::schema::Type::I64 => FieldType::I64,
      tv::schema::Type::F64 => FieldType::F64,
      tv::schema::Type::Bool => FieldType::Bool,
      tv::schema::Type::Date => FieldType::Date,
      tv::schema::Type::Facet => FieldType::Facet,
      tv::schema::Type::Bytes => FieldType::Bytes,
      tv::schema::Type::Json => FieldType::JsonObject,
      tv::schema::Type::IpAddr => FieldType::IpAddr,
    }
  }
}

/// Tantivy schema.
///
/// The schema is very strict. To build the schema the `SchemaBuilder` class is
/// provided.
#[napi]
pub struct Schema {
  pub(crate) inner: TantivySchema,
}

#[napi]
impl Schema {
  /// Get a JSON representation of the schema
  #[napi(js_name = "toJSON")]
  pub fn to_json(&self) -> String {
    serde_json::to_string(&self.inner).unwrap_or_else(|_| "{}".to_string())
  }

  /// Create a schema from JSON
  #[napi]
  pub fn from_json(json: String) -> napi::Result<Schema> {
    let schema: TantivySchema = serde_json::from_str(&json)
      .map_err(|e| napi::Error::new(napi::Status::InvalidArg, format!("Invalid JSON: {}", e)))?;
    Ok(Schema::new(schema))
  }

  /// Get the number of fields in the schema
  #[napi]
  pub fn num_fields(&self) -> u32 {
    self.inner.num_fields() as u32
  }

  /// Get field names in the schema
  #[napi]
  pub fn field_names(&self) -> Vec<String> {
    self
      .inner
      .fields()
      .map(|(_, field)| field.name().to_string())
      .collect()
  }

  /// Get field type by name
  #[napi]
  pub fn get_field_type(&self, field_name: String) -> napi::Result<FieldType> {
    let field = self.inner.get_field(&field_name).map_err(|_| {
      napi::Error::new(
        napi::Status::InvalidArg,
        format!("Field '{}' not found", field_name),
      )
    })?;
    let field_entry = self.inner.get_field_entry(field);
    Ok(FieldType::from_tantivy_type(
      &field_entry.field_type().value_type(),
    ))
  }

  /// Check if a field exists in the schema
  #[napi]
  pub fn has_field(&self, field_name: String) -> bool {
    self.inner.get_field(&field_name).is_ok()
  }

  /// Get a string representation of the schema
  #[napi]
  pub fn to_string(&self) -> String {
    format!("{:?}", self.inner)
  }
}

impl Schema {
  pub(crate) fn new(schema: TantivySchema) -> Self {
    Self { inner: schema }
  }
}
