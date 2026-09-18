use napi::{Error, Result, Status};
use napi_derive::napi;
use tantivy as tv;

use crate::to_napi_error;

fn to_json(value: &impl serde::Serialize) -> Result<serde_json::Value> {
  serde_json::to_value(value).map_err(to_napi_error)
}

/// Parse a query string into an abstract syntax tree (AST).
///
/// This function parses a query string following Tantivy's query language
/// syntax and returns a plain object representing the parsed AST.
/// Unlike `Index.parseQuery()`, this function does not require a schema
/// and returns the raw syntax tree structure.
///
/// @param query - The query string to parse.
/// @returns An object representing the parsed query AST.
///
/// @throws if the query has invalid syntax.
///
/// Example:
/// ```javascript
/// const ast = parseQuery('title:hello AND body:world')
/// ```
#[napi]
pub fn parse_query(query: String) -> Result<serde_json::Value> {
  let ast = tv::query_grammar::parse_query(&query)
    .map_err(|e| Error::new(Status::InvalidArg, format!("Query parsing error: {:?}", e)))?;
  to_json(&ast)
}

/// Parse a query string leniently, recovering from syntax errors.
///
/// This function attempts to parse a query string even if it contains
/// syntax errors. It returns both the parsed AST and a list of errors
/// encountered during parsing. Unlike `Index.parseQueryLenient()`, this
/// function does not require a schema and returns the raw syntax tree
/// structure.
///
/// @param query - The query string to parse.
/// @returns A tuple of the parsed AST and a list of syntax errors.
///
/// Example:
/// ```javascript
/// const [ast, errors] = parseQueryLenient('title:hello AND invalid:')
/// ```
#[napi]
pub fn parse_query_lenient(query: String) -> Result<(serde_json::Value, serde_json::Value)> {
  let (ast, errors) = tv::query_grammar::parse_query_lenient(&query);
  Ok((to_json(&ast)?, to_json(&errors)?))
}
