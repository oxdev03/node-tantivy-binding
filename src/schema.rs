use napi_derive::napi;
use tantivy::schema::Schema as TantivySchema;
use tantivy as tv;

/// Tantivy's FieldType
#[napi]
#[derive(PartialEq)]
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
    // Empty implementation to match tantivy-py which has no public methods
}

impl Schema {
    pub(crate) fn new(schema: TantivySchema) -> Self {
        Self { inner: schema }
    }
}
