use std::{
  convert::TryFrom,
  net::AddrParseError,
  num::{IntErrorKind, ParseFloatError, ParseIntError},
  str::ParseBoolError,
};

use napi_derive::napi;
use tantivy::{self as tv};

// TODO: Expose this to bindings once trait support is available.
pub(crate) trait QueryParserError {
  fn full_message(&self) -> String;
}

/// Error in the query syntax.
#[napi]
#[derive(Clone)]
pub struct SyntaxError {
  message: String,
}

#[napi]
impl SyntaxError {
  #[napi(getter)]
  pub fn inner_message(&self) -> String {
    self.message.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for SyntaxError {
  fn full_message(&self) -> String {
    format!("Syntax Error: {0}", self.message)
  }
}

impl From<SyntaxError> for tv::query::QueryParserError {
  fn from(error: SyntaxError) -> Self {
    tv::query::QueryParserError::SyntaxError(error.message)
  }
}

impl TryFrom<tv::query::QueryParserError> for SyntaxError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::SyntaxError(message) => Ok(Self { message }),
      _ => Err(format!("{error} is not a SyntaxError")),
    }
  }
}

/// This query is unsupported.
#[napi]
#[derive(Clone)]
pub struct UnsupportedQueryError {
  message: String,
}

#[napi]
impl UnsupportedQueryError {
  #[napi(getter)]
  pub fn inner_message(&self) -> String {
    self.message.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for UnsupportedQueryError {
  fn full_message(&self) -> String {
    format!("Unsupported query: {0}", self.message)
  }
}

impl From<UnsupportedQueryError> for tv::query::QueryParserError {
  fn from(error: UnsupportedQueryError) -> Self {
    tv::query::QueryParserError::SyntaxError(error.message)
  }
}

impl TryFrom<tv::query::QueryParserError> for UnsupportedQueryError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::UnsupportedQuery(message) => Ok(Self { message }),
      _ => Err(format!("{error} is not an UnsupportedQuery error")),
    }
  }
}

/// The query references a field that is not in the schema.
#[napi]
#[derive(Clone)]
pub struct FieldDoesNotExistError {
  field: String,
}

#[napi]
impl FieldDoesNotExistError {
  /// The name of the field causing the error.
  #[napi(getter)]
  pub fn field(&self) -> String {
    self.field.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for FieldDoesNotExistError {
  fn full_message(&self) -> String {
    format!("Field does not exist: '{0}'", self.field)
  }
}

impl From<FieldDoesNotExistError> for tv::query::QueryParserError {
  fn from(error: FieldDoesNotExistError) -> Self {
    tv::query::QueryParserError::FieldDoesNotExist(error.field)
  }
}

impl TryFrom<tv::query::QueryParserError> for FieldDoesNotExistError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::FieldDoesNotExist(field) => Ok(Self { field }),
      _ => Err(format!("{error} is not a FieldDoesNotExist error")),
    }
  }
}

/// The query contains a term for a `u64` or `i64`-field, but the value is neither.
#[napi]
#[derive(Clone)]
pub struct ExpectedIntError {
  parse_int_error: ParseIntError,
}

#[napi]
impl ExpectedIntError {
  /// If `true`, the value being parsed was empty.
  #[napi]
  pub fn caused_by_empty(&self) -> bool {
    self.parse_int_error.kind() == &IntErrorKind::Empty
  }

  /// If `true`, an invalid digit was found.
  #[napi]
  pub fn caused_by_invalid_digit(&self) -> bool {
    self.parse_int_error.kind() == &IntErrorKind::InvalidDigit
  }

  /// If `true`, the value being parsed was too large.
  #[napi]
  pub fn caused_by_pos_overflow(&self) -> bool {
    self.parse_int_error.kind() == &IntErrorKind::PosOverflow
  }

  /// If `true`, the value being parsed was too small.
  #[napi]
  pub fn caused_by_neg_overflow(&self) -> bool {
    self.parse_int_error.kind() == &IntErrorKind::NegOverflow
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for ExpectedIntError {
  fn full_message(&self) -> String {
    format!("Expected a valid integer: '{0:?}'", self.parse_int_error)
  }
}

impl From<ExpectedIntError> for tv::query::QueryParserError {
  fn from(error: ExpectedIntError) -> Self {
    tv::query::QueryParserError::ExpectedInt(error.parse_int_error)
  }
}

impl TryFrom<tv::query::QueryParserError> for ExpectedIntError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::ExpectedInt(parse_int_error) => Ok(Self { parse_int_error }),
      _ => Err(format!("{error} is not an ExpectedInt error")),
    }
  }
}

/// The query contains a term for a bytes field, but the value is not valid base64.
#[napi]
#[derive(Clone)]
pub struct ExpectedBase64Error {
  // Store error message instead of decode_error to support Clone
  message: String,
}

#[napi]
impl ExpectedBase64Error {
  /// If `true`, an invalid byte was found in the query. Padding characters (`=`) interspersed in
  /// the encoded form will be treated as invalid bytes.
  #[napi]
  pub fn caused_by_invalid_byte(&self) -> bool {
    self.message.contains("InvalidByte")
  }

  /// If the error was caused by an invalid byte, returns the offset and offending byte.
  #[napi]
  pub fn invalid_byte_info(&self) -> Option<Vec<u32>> {
    // Cannot provide detailed info without the original error
    None
  }

  /// If `true`, the length of the base64 string was invalid.
  #[napi]
  pub fn caused_by_invalid_length(&self) -> bool {
    self.message.contains("InvalidLength")
  }

  /// The last non-padding input symbol's encoded 6 bits have nonzero bits that will be discarded.
  /// If `true`, this is indicative of corrupted or truncated Base64.
  #[napi]
  pub fn caused_by_invalid_last_symbol(&self) -> bool {
    self.message.contains("InvalidLastSymbol")
  }

  /// If the error was caused by an invalid last symbol, returns the offset and offending byte.
  #[napi]
  pub fn invalid_last_symbol_info(&self) -> Option<Vec<u32>> {
    // Cannot provide detailed info without the original error
    None
  }

  /// The nature of the padding was not as configured: absent or incorrect when it must be
  /// canonical, or present when it must be absent, etc.
  #[napi]
  pub fn caused_by_invalid_padding(&self) -> bool {
    self.message.contains("InvalidPadding")
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for ExpectedBase64Error {
  fn full_message(&self) -> String {
    format!("Expected base64: {}", self.message)
  }
}

impl From<ExpectedBase64Error> for tv::query::QueryParserError {
  fn from(error: ExpectedBase64Error) -> Self {
    // We can't reconstruct the original DecodeError, so use a generic one
    tv::query::QueryParserError::SyntaxError(error.message)
  }
}

impl TryFrom<tv::query::QueryParserError> for ExpectedBase64Error {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::ExpectedBase64(decode_error) => Ok(Self {
        message: format!("{:?}", decode_error),
      }),
      _ => Err(format!("{error} is not an ExpectedBase64 error")),
    }
  }
}

/// The query contains a term for a `f64`-field, but the value is not a f64.
#[napi]
#[derive(Clone)]
pub struct ExpectedFloatError {
  parse_float_error: ParseFloatError,
}

#[napi]
impl ExpectedFloatError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for ExpectedFloatError {
  fn full_message(&self) -> String {
    format!("Expected a float value: '{0:?}'", self.parse_float_error)
  }
}

impl From<ExpectedFloatError> for tv::query::QueryParserError {
  fn from(error: ExpectedFloatError) -> Self {
    tv::query::QueryParserError::ExpectedFloat(error.parse_float_error)
  }
}

impl TryFrom<tv::query::QueryParserError> for ExpectedFloatError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::ExpectedFloat(parse_float_error) => {
        Ok(Self { parse_float_error })
      }
      _ => Err(format!("{error} is not an ExpectedFloat error")),
    }
  }
}

/// The query contains a term for a `bool`-field, but the value is not a bool.
#[napi]
#[derive(Clone)]
pub struct ExpectedBoolError {
  parse_bool_error: ParseBoolError,
}

#[napi]
impl ExpectedBoolError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for ExpectedBoolError {
  fn full_message(&self) -> String {
    format!("Expected a bool value: '{0:?}'", self.parse_bool_error)
  }
}

impl From<ExpectedBoolError> for tv::query::QueryParserError {
  fn from(error: ExpectedBoolError) -> Self {
    tv::query::QueryParserError::ExpectedBool(error.parse_bool_error)
  }
}

impl TryFrom<tv::query::QueryParserError> for ExpectedBoolError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::ExpectedBool(parse_bool_error) => Ok(Self { parse_bool_error }),
      _ => Err(format!("{error} is not an ExpectedBool error")),
    }
  }
}

/// It is forbidden queries that are only "excluding". (e.g. -title:pop)
#[napi]
#[derive(Clone)]
pub struct AllButQueryForbiddenError;

#[napi]
impl AllButQueryForbiddenError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for AllButQueryForbiddenError {
  fn full_message(&self) -> String {
    "Invalid query: Only excluding terms given".to_string()
  }
}

impl From<AllButQueryForbiddenError> for tv::query::QueryParserError {
  fn from(_error: AllButQueryForbiddenError) -> Self {
    tv::query::QueryParserError::AllButQueryForbidden
  }
}

impl TryFrom<tv::query::QueryParserError> for AllButQueryForbiddenError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::AllButQueryForbidden => Ok(Self {}),
      _ => Err(format!("{error} is not an AllButQueryForbidden error")),
    }
  }
}

/// If no default field is declared, running a query without any field specified is forbbidden.
#[napi]
#[derive(Clone)]
pub struct NoDefaultFieldDeclaredError;

#[napi]
impl NoDefaultFieldDeclaredError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for NoDefaultFieldDeclaredError {
  fn full_message(&self) -> String {
    "No default field declared and no field specified in query".to_string()
  }
}

impl From<NoDefaultFieldDeclaredError> for tv::query::QueryParserError {
  fn from(_error: NoDefaultFieldDeclaredError) -> Self {
    tv::query::QueryParserError::NoDefaultFieldDeclared
  }
}

impl TryFrom<tv::query::QueryParserError> for NoDefaultFieldDeclaredError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::NoDefaultFieldDeclared => Ok(Self {}),
      _ => Err(format!("{error} is not a NoDefaultFieldDeclared error")),
    }
  }
}

/// The field searched for is not declared as indexed in the schema.
#[napi]
#[derive(Clone)]
pub struct FieldNotIndexedError {
  field: String,
}

#[napi]
impl FieldNotIndexedError {
  #[napi]
  pub fn field(&self) -> String {
    self.field.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for FieldNotIndexedError {
  fn full_message(&self) -> String {
    format!("The field '{0}' is not declared as indexed", self.field)
  }
}

impl From<FieldNotIndexedError> for tv::query::QueryParserError {
  fn from(error: FieldNotIndexedError) -> Self {
    tv::query::QueryParserError::FieldNotIndexed(error.field)
  }
}

impl TryFrom<tv::query::QueryParserError> for FieldNotIndexedError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::FieldNotIndexed(field) => Ok(Self { field }),
      _ => Err(format!("{error} is not an FieldNotIndexed error")),
    }
  }
}

/// A phrase query was requested for a field that does not have any positions indexed.
#[napi]
#[derive(Clone)]
pub struct FieldDoesNotHavePositionsIndexedError {
  field: String,
}

#[napi]
impl FieldDoesNotHavePositionsIndexedError {
  #[napi]
  pub fn field(&self) -> String {
    self.field.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for FieldDoesNotHavePositionsIndexedError {
  fn full_message(&self) -> String {
    format!(
      "The field '{0}' does not have positions indexed",
      self.field
    )
  }
}

impl From<FieldDoesNotHavePositionsIndexedError> for tv::query::QueryParserError {
  fn from(error: FieldDoesNotHavePositionsIndexedError) -> Self {
    tv::query::QueryParserError::FieldDoesNotHavePositionsIndexed(error.field)
  }
}

impl TryFrom<tv::query::QueryParserError> for FieldDoesNotHavePositionsIndexedError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::FieldDoesNotHavePositionsIndexed(field) => Ok(Self { field }),
      _ => Err(format!(
        "{error} is not a FieldDoesNotHavePositionsIndexed error"
      )),
    }
  }
}

/// A phrase-prefix query requires at least two terms
#[napi]
#[derive(Clone)]
pub struct PhrasePrefixRequiresAtLeastTwoTermsError {
  /// The phrase which triggered the issue.
  phrase: String,
  /// The tokenizer configured for the field.
  tokenizer: String,
}

#[napi]
impl PhrasePrefixRequiresAtLeastTwoTermsError {
  #[napi]
  pub fn phrase(&self) -> String {
    self.phrase.clone()
  }

  #[napi]
  pub fn tokenizer(&self) -> String {
    self.tokenizer.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for PhrasePrefixRequiresAtLeastTwoTermsError {
  fn full_message(&self) -> String {
    format!(
      "The phrase '{0:?}' does not produce at least two terms using the tokenizer '{1:?}'",
      self.phrase, self.tokenizer
    )
  }
}

impl From<PhrasePrefixRequiresAtLeastTwoTermsError> for tv::query::QueryParserError {
  fn from(error: PhrasePrefixRequiresAtLeastTwoTermsError) -> Self {
    tv::query::QueryParserError::PhrasePrefixRequiresAtLeastTwoTerms {
      phrase: error.phrase,
      tokenizer: error.tokenizer,
    }
  }
}

impl TryFrom<tv::query::QueryParserError> for PhrasePrefixRequiresAtLeastTwoTermsError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::PhrasePrefixRequiresAtLeastTwoTerms { phrase, tokenizer } => {
        Ok(Self { phrase, tokenizer })
      }
      _ => Err(format!(
        "{error} is not a PhrasePrefixRequiresAtLeastTwoTerms error"
      )),
    }
  }
}

/// The tokenizer for the given field is unknown.
#[napi]
#[derive(Clone)]
pub struct UnknownTokenizerError {
  /// The name of the tokenizer.
  tokenizer: String,
  /// The field name.
  field: String,
}

#[napi]
impl UnknownTokenizerError {
  #[napi]
  pub fn tokenizer(&self) -> String {
    self.tokenizer.clone()
  }

  #[napi]
  pub fn field(&self) -> String {
    self.field.clone()
  }

  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for UnknownTokenizerError {
  fn full_message(&self) -> String {
    format!(
      "The tokenizer '{0:?}' for the field '{1:?}' is unknown",
      self.tokenizer, self.field
    )
  }
}

impl From<UnknownTokenizerError> for tv::query::QueryParserError {
  fn from(error: UnknownTokenizerError) -> Self {
    tv::query::QueryParserError::UnknownTokenizer {
      tokenizer: error.tokenizer,
      field: error.field,
    }
  }
}

impl TryFrom<tv::query::QueryParserError> for UnknownTokenizerError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::UnknownTokenizer { tokenizer, field } => {
        Ok(Self { tokenizer, field })
      }
      _ => Err(format!("{error} is not an UnknownTokenizer error")),
    }
  }
}

/// The query contains a range query with a phrase as one of the bounds. Only terms can be used as
/// bounds.
#[napi]
#[derive(Clone)]
pub struct RangeMustNotHavePhraseError;

#[napi]
impl RangeMustNotHavePhraseError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for RangeMustNotHavePhraseError {
  fn full_message(&self) -> String {
    "A range query cannot have a phrase as one of the bounds".to_string()
  }
}

impl From<RangeMustNotHavePhraseError> for tv::query::QueryParserError {
  fn from(_error: RangeMustNotHavePhraseError) -> Self {
    tv::query::QueryParserError::RangeMustNotHavePhrase
  }
}

impl TryFrom<tv::query::QueryParserError> for RangeMustNotHavePhraseError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::RangeMustNotHavePhrase => Ok(Self {}),
      _ => Err(format!("{error} is not a RangeMustNotHavePhrase error")),
    }
  }
}

/// The format for the date field is not RFC 3339 compliant.
#[napi]
#[derive(Clone)]
pub struct DateFormatError {
  // Store error message instead of full error to support Clone
  message: String,
}

#[napi]
impl DateFormatError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for DateFormatError {
  fn full_message(&self) -> String {
    format!("The date field has an invalid format: {}", self.message)
  }
}

impl From<DateFormatError> for tv::query::QueryParserError {
  fn from(error: DateFormatError) -> Self {
    // We can't reconstruct the original DateFormatError, so use a generic one
    tv::query::QueryParserError::SyntaxError(error.message)
  }
}

impl TryFrom<tv::query::QueryParserError> for DateFormatError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::DateFormatError(_) => Ok(Self {
        message: format!("{}", error),
      }),
      _ => Err(format!("{error} is not a DateFormatError")),
    }
  }
}

/// The format for the facet field is invalid.
#[napi]
#[derive(Clone)]
pub struct FacetFormatError {
  // Store error message instead of facet_parse_error to support Clone
  message: String,
}

#[napi]
impl FacetFormatError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for FacetFormatError {
  fn full_message(&self) -> String {
    format!("The facet field is malformed: {}", self.message)
  }
}

impl From<FacetFormatError> for tv::query::QueryParserError {
  fn from(error: FacetFormatError) -> Self {
    // We can't reconstruct the original FacetParseError, so use a generic one
    tv::query::QueryParserError::SyntaxError(error.message)
  }
}

impl TryFrom<tv::query::QueryParserError> for FacetFormatError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::FacetFormatError(facet_parse_error) => Ok(Self {
        message: format!("{}", facet_parse_error),
      }),
      _ => Err(format!("{error} is not a FacetFormatError")),
    }
  }
}

/// The format for the ip field is invalid.
#[napi]
#[derive(Clone)]
pub struct IpFormatError {
  addr_parse_error: AddrParseError,
}

#[napi]
impl IpFormatError {
  #[napi(js_name = "toString")]
  pub fn to_string(&self) -> String {
    self.full_message()
  }
}

impl QueryParserError for IpFormatError {
  fn full_message(&self) -> String {
    format!("The facet field is malformed: {0}", self.addr_parse_error)
  }
}

impl From<IpFormatError> for tv::query::QueryParserError {
  fn from(error: IpFormatError) -> Self {
    tv::query::QueryParserError::IpFormatError(error.addr_parse_error)
  }
}

impl TryFrom<tv::query::QueryParserError> for IpFormatError {
  type Error = String;

  fn try_from(error: tv::query::QueryParserError) -> std::result::Result<Self, Self::Error> {
    match error {
      tv::query::QueryParserError::IpFormatError(addr_parse_error) => Ok(Self { addr_parse_error }),
      _ => Err(format!("{error} is not an IpFormatError")),
    }
  }
}
