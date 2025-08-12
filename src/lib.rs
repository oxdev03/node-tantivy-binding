use napi_derive::napi;
use napi::{Error, JsUnknown, Result, Status};
use tantivy as tv;

/// Get the version of the library
#[napi]
pub fn get_version() -> String {
    "0.1.0".to_string()
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
            format!("Field '{}' is not defined in the schema.", field_name),
        )
    })
}

pub(crate) fn make_term(
    schema: &tv::schema::Schema,
    field_name: &str,
    field_value: JsUnknown,
) -> Result<tv::Term> {
    let field = get_field(schema, field_name)?;
    let field_entry = schema.get_field_entry(field);
    let field_type = crate::schema::FieldType::from_tantivy_type(&field_entry.field_type().value_type());
    
    match field_type {
        crate::schema::FieldType::Str => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_text(field, &str_val))
        },
        crate::schema::FieldType::U64 => {
            let num_val = field_value.coerce_to_number()?.get_uint32()? as u64;
            Ok(tv::Term::from_field_u64(field, num_val))
        },
        crate::schema::FieldType::I64 => {
            let num_val = field_value.coerce_to_number()?.get_int64()?;
            Ok(tv::Term::from_field_i64(field, num_val))
        },
        crate::schema::FieldType::F64 => {
            let num_val = field_value.coerce_to_number()?.get_double()?;
            Ok(tv::Term::from_field_f64(field, num_val))
        },
        crate::schema::FieldType::Date => {
            let num_val = field_value.coerce_to_number()?.get_int64()?;
            Ok(tv::Term::from_field_date(field, tv::DateTime::from_timestamp_secs(num_val)))
        },
        crate::schema::FieldType::Facet => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            let facet = tv::schema::Facet::from(&str_val);
            Ok(tv::Term::from_facet(field, &facet))
        },
        crate::schema::FieldType::Bytes => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_bytes(field, str_val.as_bytes()))
        },
        crate::schema::FieldType::Bool => {
            let bool_val = field_value.coerce_to_bool()?.get_value()?;
            Ok(tv::Term::from_field_bool(field, bool_val))
        },
        crate::schema::FieldType::IpAddr => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            let ip_addr: std::net::IpAddr = str_val.parse()
                .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid IP address: {}", e)))?;
            // Convert IpAddr to Ipv6Addr for Tantivy
            let ipv6_addr = match ip_addr {
                std::net::IpAddr::V6(v6) => v6,
                std::net::IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            };
            Ok(tv::Term::from_field_ip_addr(field, ipv6_addr))
        },
        crate::schema::FieldType::JsonObject => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_json_path(field, &str_val, false))
        }
    }
}

pub(crate) fn make_term_for_type(
    schema: &tv::schema::Schema,
    field_name: &str,
    field_type: crate::schema::FieldType,
    field_value: JsUnknown,
) -> Result<tv::Term> {
    let field = get_field(schema, field_name)?;
    
    match field_type {
        crate::schema::FieldType::Str => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_text(field, &str_val))
        },
        crate::schema::FieldType::U64 => {
            let num_val = field_value.coerce_to_number()?.get_uint32()? as u64;
            Ok(tv::Term::from_field_u64(field, num_val))
        },
        crate::schema::FieldType::I64 => {
            let num_val = field_value.coerce_to_number()?.get_int64()?;
            Ok(tv::Term::from_field_i64(field, num_val))
        },
        crate::schema::FieldType::F64 => {
            let num_val = field_value.coerce_to_number()?.get_double()?;
            Ok(tv::Term::from_field_f64(field, num_val))
        },
        crate::schema::FieldType::Date => {
            let num_val = field_value.coerce_to_number()?.get_int64()?;
            Ok(tv::Term::from_field_date(field, tv::DateTime::from_timestamp_secs(num_val)))
        },
        crate::schema::FieldType::Facet => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            let facet = tv::schema::Facet::from(&str_val);
            Ok(tv::Term::from_facet(field, &facet))
        },
        crate::schema::FieldType::Bytes => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_bytes(field, str_val.as_bytes()))
        },
        crate::schema::FieldType::Bool => {
            let bool_val = field_value.coerce_to_bool()?.get_value()?;
            Ok(tv::Term::from_field_bool(field, bool_val))
        },
        crate::schema::FieldType::IpAddr => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            let ip_addr: std::net::IpAddr = str_val.parse()
                .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid IP address: {}", e)))?;
            // Convert IpAddr to Ipv6Addr for Tantivy
            let ipv6_addr = match ip_addr {
                std::net::IpAddr::V6(v6) => v6,
                std::net::IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            };
            Ok(tv::Term::from_field_ip_addr(field, ipv6_addr))
        },
        crate::schema::FieldType::JsonObject => {
            let str_val = field_value.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(tv::Term::from_field_json_path(field, &str_val, false))
        }
    }
}

// Start with just the schema builder and schema
pub mod schemabuilder;
pub mod schema;
pub mod document;
pub mod query;
pub mod searcher;
pub mod snippet;
pub mod tokenizer;
pub mod parser_error;
pub mod index;
pub use schemabuilder::SchemaBuilder;
pub use schema::{Schema, FieldType};
pub use document::Document;
pub use query::{Query, Occur};
pub use searcher::Searcher;
pub use snippet::{Snippet, SnippetGenerator};
pub use tokenizer::{TokenizerStatic, FilterStatic, Tokenizer, Filter, TextAnalyzer, TextAnalyzerBuilder};
pub use parser_error::{
    SyntaxError, UnsupportedQueryError, FieldDoesNotExistError, ExpectedIntError,
    ExpectedFloatError, ExpectedBoolError, AllButQueryForbiddenError,
    NoDefaultFieldDeclaredError, FieldNotIndexedError, FieldDoesNotHavePositionsIndexedError,
    PhrasePrefixRequiresAtLeastTwoTermsError, UnknownTokenizerError, RangeMustNotHavePhraseError,
    DateFormatError, FacetFormatError, IpFormatError
};
pub use index::{Index, IndexWriter};
