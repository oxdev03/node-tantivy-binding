use napi::bindgen_prelude::*;
use napi::{Error, Result, Status};
use napi_derive::napi;
use tantivy::{self as tv, schema::OwnedValue as Value};

/// Get the version of the library
#[napi]
pub fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

/// Get the version of the underlying tantivy engine.
#[napi]
pub fn get_tantivy_version() -> String {
  tv::version_string().to_string()
}

// Helper functions for query operations
pub(crate) fn to_napi_error(e: impl std::error::Error) -> Error {
  Error::new(Status::GenericFailure, format!("{}", e))
}

pub(crate) fn get_field(
  schema: &tv::schema::Schema,
  field_name: &str,
) -> Result<tv::schema::Field> {
  schema.get_field(field_name).map_err(|_| {
    Error::new(
      Status::InvalidArg,
      format!("Field `{field_name}` is not defined in the schema."),
    )
  })
}

pub(crate) fn make_term(
  schema: &tv::schema::Schema,
  field_name: &str,
  field_value: &Unknown,
) -> Result<tv::Term> {
  let field = get_field(schema, field_name)?;
  // Look up the actual field type from the schema so that JavaScript numbers
  // are extracted as the correct numeric type (u64 vs i64). The generic
  // `extract_value()` path infers integers from the runtime value alone, which
  // silently produces wrong terms for u64 fields.
  let field_type = schema.get_field_entry(field).field_type().value_type();
  let value = crate::document::extract_value_for_type(field_value, field_type, field_name)?;
  term_from_value(field, field_name, value)
}

pub(crate) fn make_term_for_type(
  schema: &tv::schema::Schema,
  field_name: &str,
  field_type: crate::schema::FieldType,
  field_value: &Unknown,
) -> Result<tv::Term> {
  let field = get_field(schema, field_name)?;
  let value = crate::document::extract_value_for_type(field_value, field_type.into(), field_name)?;
  term_from_value(field, field_name, value)
}

fn term_from_value(field: tv::schema::Field, field_name: &str, value: Value) -> Result<tv::Term> {
  Ok(match value {
    Value::Str(text) => tv::Term::from_field_text(field, &text),
    Value::U64(num) => tv::Term::from_field_u64(field, num),
    Value::I64(num) => tv::Term::from_field_i64(field, num),
    Value::F64(num) => tv::Term::from_field_f64(field, num),
    Value::Date(d) => tv::Term::from_field_date(field, d),
    Value::Facet(facet) => tv::Term::from_facet(field, &facet),
    Value::Bool(b) => tv::Term::from_field_bool(field, b),
    Value::IpAddr(i) => tv::Term::from_field_ip_addr(field, i),
    Value::Bytes(ref bytes) => tv::Term::from_field_bytes(field, bytes),
    _ => {
      return Err(Error::new(
        Status::InvalidArg,
        format!("Can't create a term for Field `{field_name}` with the given value."),
      ))
    }
  })
}

pub mod document;
pub mod explanation;
pub mod facet;
pub mod index;
pub mod parser_error;
pub mod query;
pub mod query_grammar;
pub mod schema;
pub mod schemabuilder;
pub mod searcher;
pub mod snippet;
pub mod tokenizer;
pub use document::Document;
pub use facet::Facet;
pub use index::{Index, IndexWriter};
pub use parser_error::{
  AllButQueryForbiddenError, DateFormatError, ExpectedBoolError, ExpectedFloatError,
  ExpectedIntError, FacetFormatError, FieldDoesNotExistError,
  FieldDoesNotHavePositionsIndexedError, FieldNotIndexedError, IpFormatError,
  NoDefaultFieldDeclaredError, PhrasePrefixRequiresAtLeastTwoTermsError,
  RangeMustNotHavePhraseError, SyntaxError, UnknownTokenizerError, UnsupportedQueryError,
};
pub use query::{Occur, Query};
pub use query_grammar::{parse_query, parse_query_lenient};
pub use schema::{FieldType, Schema};
pub use schemabuilder::SchemaBuilder;
pub use searcher::Searcher;
pub use snippet::{Snippet, SnippetGenerator};
pub use tokenizer::{
  Filter, FilterStatic, TextAnalyzer, TextAnalyzerBuilder, Tokenizer, TokenizerStatic,
};
