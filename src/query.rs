use crate::{
  document::Document, explanation::Explanation, get_field, make_term, make_term_for_type,
  schema::FieldType, searcher::DocAddress, to_napi_error, Schema,
};
use core::ops::Bound as OpsBound;
use napi::bindgen_prelude::*;
use napi::{Error, Result, Status};
use napi_derive::napi;
use tantivy as tv;

/// Represents a Tantivy Occur type for BooleanQuery
#[napi]
pub enum Occur {
  Must,
  Should,
  MustNot,
}

impl From<Occur> for tv::query::Occur {
  fn from(occur: Occur) -> tv::query::Occur {
    match occur {
      Occur::Must => tv::query::Occur::Must,
      Occur::Should => tv::query::Occur::Should,
      Occur::MustNot => tv::query::Occur::MustNot,
    }
  }
}

/// Tantivy's Query
#[napi]
pub struct Query {
  pub(crate) inner: Box<dyn tv::query::Query>,
}

impl Clone for Query {
  fn clone(&self) -> Self {
    Query {
      inner: self.inner.box_clone(),
    }
  }
}

impl Query {
  pub(crate) fn get(&self) -> &dyn tv::query::Query {
    &self.inner
  }

  // Arguments mirror tantivy's MoreLikeThisQuery builder options one-to-one;
  // a config struct here would only duplicate that builder.
  #[allow(clippy::too_many_arguments)]
  fn more_like_this_builder(
    min_doc_frequency: Option<f64>,
    max_doc_frequency: Option<f64>,
    min_term_frequency: Option<u32>,
    max_query_terms: Option<u32>,
    min_word_length: Option<u32>,
    max_word_length: Option<u32>,
    boost_factor: Option<f64>,
    stop_words: Option<Vec<String>>,
  ) -> tv::query::MoreLikeThisQueryBuilder {
    let mut builder = tv::query::MoreLikeThisQuery::builder();
    if let Some(value) = min_doc_frequency {
      builder = builder.with_min_doc_frequency(value as u64);
    }
    if let Some(value) = max_doc_frequency {
      builder = builder.with_max_doc_frequency(value as u64);
    }
    if let Some(value) = min_term_frequency {
      builder = builder.with_min_term_frequency(value as usize);
    }
    if let Some(value) = max_query_terms {
      builder = builder.with_max_query_terms(value as usize);
    }
    if let Some(value) = min_word_length {
      builder = builder.with_min_word_length(value as usize);
    }
    if let Some(value) = max_word_length {
      builder = builder.with_max_word_length(value as usize);
    }
    if let Some(value) = boost_factor {
      builder = builder.with_boost_factor(value as f32);
    }
    builder.with_stop_words(stop_words.unwrap_or_default())
  }

  /// This is an internal helper for the BooleanQuery convenience methods
  /// (`andMustMatch`, `orShouldMatch`, `andMustNotMatch`). It builds a new
  /// query in which each query in `others` is added as an `other_occur`
  /// clause alongside `self`.
  ///
  /// When `self` is already a BooleanQuery, the new clauses are appended to
  /// its clause list where doing so preserves matching semantics, so that
  /// fluent chains stay flat instead of nesting one level per call:
  ///
  /// - Must/MustNot clauses can always be appended, provided the existing
  ///   `minimum_number_should_match` is carried over. (Tantivy recomputes
  ///   the minimum to 0 when a Must/MustNot clause is present, which would
  ///   silently turn existing Should clauses from required-disjunction into
  ///   optional scoring hints.)
  /// - Should clauses can only be appended when the existing query is a
  ///   plain disjunction: all clauses Should, with the default minimum of 1.
  ///   In any other case, e.g. `a.andMustMatch(b).orShouldMatch(c)`,
  ///   appending would change which documents match.
  ///
  /// In all other cases `self` is nested as a single `self_occur` clause of
  /// a new BooleanQuery.
  fn combine_with(
    &self,
    others: Vec<&Query>,
    self_occur: tv::query::Occur,
    other_occur: tv::query::Occur,
  ) -> Query {
    use tv::query::BooleanQuery;
    use tv::query::Occur;

    if others.is_empty() {
      return self.clone();
    }
    let new_clauses = others
      .into_iter()
      .map(|query| (other_occur, query.inner.box_clone()));

    if let Some(boolean_query) = self.inner.downcast_ref::<BooleanQuery>() {
      let minimum = boolean_query.get_minimum_number_should_match();
      let appendable = match other_occur {
        Occur::Must | Occur::MustNot => true,
        Occur::Should => {
          minimum == 1
            && boolean_query
              .clauses()
              .iter()
              .all(|(occur, _)| *occur == Occur::Should)
        }
      };
      if appendable {
        let mut subqueries = boolean_query
          .clauses()
          .iter()
          .map(|(occur, subquery)| (*occur, subquery.box_clone()))
          .collect::<Vec<_>>();
        subqueries.extend(new_clauses);
        return Query {
          inner: Box::new(BooleanQuery::with_minimum_required_clauses(
            subqueries, minimum,
          )),
        };
      }
    }

    let mut subqueries = vec![(self_occur, self.inner.box_clone())];
    subqueries.extend(new_clauses);
    Query {
      inner: Box::new(BooleanQuery::new(subqueries)),
    }
  }
}

fn index_record_option(index_option: &str) -> Result<tv::schema::IndexRecordOption> {
  match index_option {
    "position" => Ok(tv::schema::IndexRecordOption::WithFreqsAndPositions),
    "freq" => Ok(tv::schema::IndexRecordOption::WithFreqs),
    "basic" => Ok(tv::schema::IndexRecordOption::Basic),
    _ => Err(Error::new(
      Status::InvalidArg,
      "Invalid index option, valid choices are: 'basic', 'freq' and 'position'",
    )),
  }
}

fn boxed(inner: impl tv::query::Query + 'static) -> Query {
  Query {
    inner: Box::new(inner),
  }
}

#[napi]
impl Query {
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    format!("Query({:?})", self.get())
  }

  /// Construct a Tantivy's TermQuery
  #[napi(factory)]
  pub fn term_query(
    schema: &Schema,
    field_name: String,
    field_value: Unknown,
    index_option: Option<String>,
  ) -> Result<Query> {
    let term = make_term(&schema.inner, &field_name, &field_value)?;
    let index_option = index_record_option(index_option.as_deref().unwrap_or("position"))?;
    Ok(boxed(tv::query::TermQuery::new(term, index_option)))
  }

  /// Construct a Tantivy's TermSetQuery
  #[napi(factory)]
  pub fn term_set_query(
    schema: &Schema,
    field_name: String,
    field_values: Vec<Unknown>,
  ) -> Result<Query> {
    let terms = field_values
      .iter()
      .map(|field_value| make_term(&schema.inner, &field_name, field_value))
      .collect::<Result<Vec<_>>>()?;
    Ok(boxed(tv::query::TermSetQuery::new(terms)))
  }

  /// Construct a Tantivy's AllQuery
  #[napi(factory)]
  pub fn all_query() -> Result<Query> {
    Ok(boxed(tv::query::AllQuery))
  }

  /// Construct a Tantivy's EmptyQuery
  ///
  /// A query that matches no documents. Useful as a placeholder or default.
  #[napi(factory)]
  pub fn empty_query() -> Result<Query> {
    Ok(boxed(tv::query::EmptyQuery))
  }

  /// Construct a Tantivy's ExistsQuery
  ///
  /// Matches all documents that have at least one non-null value in the given
  /// field. Executing a search with this query will fail if the field doesn't
  /// exist or is not a fast field.
  ///
  /// @param fastFieldName - Field name to be searched.
  /// @param jsonSubpaths - If true, check all the subpaths inside a JSON field.
  #[napi(factory)]
  pub fn exists_query(fast_field_name: String, json_subpaths: Option<bool>) -> Result<Query> {
    Ok(boxed(tv::query::ExistsQuery::new(
      fast_field_name,
      json_subpaths.unwrap_or(false),
    )))
  }

  /// Construct a Tantivy's FuzzyTermQuery
  ///
  /// @param schema - Schema of the target index.
  /// @param fieldName - Field name to be searched.
  /// @param text - String representation of the query term.
  /// @param distance - (Optional) Edit distance you are going to allow. When not specified, the default is 1.
  /// @param transpositionCostOne - (Optional) If true, a transposition (swapping) cost will be 1; otherwise it will be 2. When not specified, the default is true.
  /// @param prefix - (Optional) If true, prefix levenshtein distance is applied. When not specified, the default is false.
  #[napi(factory)]
  pub fn fuzzy_term_query(
    schema: &Schema,
    field_name: String,
    text: String,
    distance: Option<u8>,
    transposition_cost_one: Option<bool>,
    prefix: Option<bool>,
  ) -> Result<Query> {
    let distance = distance.unwrap_or(1);
    let transposition_cost_one = transposition_cost_one.unwrap_or(true);

    let field = get_field(&schema.inner, &field_name)?;
    let term = tv::Term::from_field_text(field, &text);
    Ok(if prefix.unwrap_or(false) {
      boxed(tv::query::FuzzyTermQuery::new_prefix(
        term,
        distance,
        transposition_cost_one,
      ))
    } else {
      boxed(tv::query::FuzzyTermQuery::new(
        term,
        distance,
        transposition_cost_one,
      ))
    })
  }

  /// Construct a Tantivy's PhraseQuery with custom offsets and slop
  ///
  /// @param schema - Schema of the target index.
  /// @param fieldName - Field name to be searched.
  /// @param words - Word list that constructs the phrase. A word can be a term
  ///         text, or a `[offset, text]` pair giving its offset in the phrase.
  /// @param slop - (Optional) The number of gaps permitted between the words in the query phrase. Default is 0.
  #[napi(factory)]
  pub fn phrase_query(
    schema: &Schema,
    field_name: String,
    words: Vec<Unknown>,
    slop: Option<u32>,
  ) -> Result<Query> {
    let terms_with_offset = phrase_words(&words, |w| make_term(&schema.inner, &field_name, w))?;
    Ok(boxed(tv::query::PhraseQuery::new_with_offset_and_slop(
      terms_with_offset,
      slop.unwrap_or(0),
    )))
  }

  /// Construct a Tantivy's PhrasePrefixQuery with custom offsets
  ///
  /// Matches a specific sequence of words followed by a term of which only a
  /// prefix is known. Requires positions to be indexed on the target field.
  ///
  /// @param schema - Schema of the target index.
  /// @param fieldName - Field name to be searched.
  /// @param words - Word list that constructs the phrase. A word can be a term
  ///         text, or a `[offset, text]` pair giving its offset in the phrase.
  #[napi(factory)]
  pub fn phrase_prefix_query(
    schema: &Schema,
    field_name: String,
    words: Vec<Unknown>,
  ) -> Result<Query> {
    let terms_with_offset = phrase_words(&words, |w| make_term(&schema.inner, &field_name, w))?;
    Ok(boxed(tv::query::PhrasePrefixQuery::new_with_offset(
      terms_with_offset,
    )))
  }

  /// Construct a Tantivy's RegexPhraseQuery
  ///
  /// Matches a specific sequence of regex patterns in positional order, with
  /// optional slop. Each pattern can match multiple indexed terms via regex
  /// expansion.
  ///
  /// @param schema - Schema of the target index.
  /// @param fieldName - Field name to be searched.
  /// @param words - Pattern list forming the phrase. A pattern can be a string,
  ///         or a `[offset, pattern]` pair giving its offset in the phrase.
  /// @param slop - (Optional) Number of gaps permitted between matched terms. Default is 0.
  #[napi(factory)]
  pub fn regex_phrase_query(
    schema: &Schema,
    field_name: String,
    words: Vec<Unknown>,
    slop: Option<u32>,
  ) -> Result<Query> {
    let field = get_field(&schema.inner, &field_name)?;
    let patterns_with_offset =
      phrase_words(&words, |w| w.coerce_to_string()?.into_utf8()?.into_owned())?;
    Ok(boxed(
      tv::query::RegexPhraseQuery::new_with_offset_and_slop(
        field,
        patterns_with_offset,
        slop.unwrap_or(0),
      ),
    ))
  }

  /// Construct a Tantivy's BooleanQuery
  ///
  /// @param subqueries - `{ occur, query }` pairs making up the clauses.
  /// @param minimumNumberShouldMatch - (Optional) How many Should clauses a
  ///         document must match. Defaults to tantivy's own rule: 1 when there
  ///         is no Must/MustNot clause, 0 otherwise.
  #[napi(factory)]
  pub fn boolean_query(
    subqueries: Vec<Object>,
    minimum_number_should_match: Option<u32>,
  ) -> Result<Query> {
    let mut dyn_subqueries = Vec::with_capacity(subqueries.len());

    for subquery_obj in subqueries {
      let occur: Occur = subquery_obj
        .get("occur")?
        .ok_or_else(|| Error::new(Status::InvalidArg, "Missing 'occur' field in subquery"))?;
      let query: ClassInstance<Query> = subquery_obj
        .get("query")?
        .ok_or_else(|| Error::new(Status::InvalidArg, "Missing 'query' field in subquery"))?;

      dyn_subqueries.push((occur.into(), query.inner.box_clone()));
    }

    Ok(match minimum_number_should_match {
      None => boxed(tv::query::BooleanQuery::from(dyn_subqueries)),
      Some(n) => boxed(tv::query::BooleanQuery::with_minimum_required_clauses(
        dyn_subqueries,
        n as usize,
      )),
    })
  }

  /// Combine queries with AND (MUST) logic.
  ///
  /// Returns a query matching documents that match this query and every
  /// given query.
  #[napi]
  pub fn and_must_match(&self, queries: Vec<&Query>) -> Query {
    self.combine_with(queries, tv::query::Occur::Must, tv::query::Occur::Must)
  }

  /// Combine queries with AND NOT (MUST NOT) logic.
  ///
  /// Returns a query matching documents that match this query and none of
  /// the given queries.
  #[napi]
  pub fn and_must_not_match(&self, queries: Vec<&Query>) -> Query {
    self.combine_with(queries, tv::query::Occur::Must, tv::query::Occur::MustNot)
  }

  /// Combine queries with OR (SHOULD) logic.
  ///
  /// Returns a query matching documents that match this query or any of the
  /// given queries.
  #[napi]
  pub fn or_should_match(&self, queries: Vec<&Query>) -> Query {
    self.combine_with(queries, tv::query::Occur::Should, tv::query::Occur::Should)
  }

  /// Construct a Tantivy's DisjunctionMaxQuery
  #[napi(factory)]
  pub fn disjunction_max_query(subqueries: Vec<&Query>, tie_breaker: Option<f64>) -> Result<Query> {
    let inner_queries: Vec<Box<dyn tv::query::Query>> = subqueries
      .iter()
      .map(|query| query.inner.box_clone())
      .collect();

    Ok(match tie_breaker {
      Some(tie_breaker) => boxed(tv::query::DisjunctionMaxQuery::with_tie_breaker(
        inner_queries,
        tie_breaker as f32,
      )),
      None => boxed(tv::query::DisjunctionMaxQuery::new(inner_queries)),
    })
  }

  /// Construct a Tantivy's BoostQuery
  #[napi(factory)]
  pub fn boost_query(query: &Query, boost: f64) -> Result<Query> {
    Ok(boxed(tv::query::BoostQuery::new(
      query.inner.box_clone(),
      boost as f32,
    )))
  }

  /// Construct a Tantivy's RegexQuery
  #[napi(factory)]
  pub fn regex_query(schema: &Schema, field_name: String, regex_pattern: String) -> Result<Query> {
    let field = get_field(&schema.inner, &field_name)?;
    tv::query::RegexQuery::from_pattern(&regex_pattern, field)
      .map(boxed)
      .map_err(to_napi_error)
  }

  /// Construct a Tantivy's MoreLikeThisQuery from an indexed document.
  #[napi(factory)]
  #[allow(clippy::too_many_arguments)]
  pub fn more_like_this_query(
    doc_address: DocAddress,
    min_doc_frequency: Option<f64>,
    max_doc_frequency: Option<f64>,
    min_term_frequency: Option<u32>,
    max_query_terms: Option<u32>,
    min_word_length: Option<u32>,
    max_word_length: Option<u32>,
    boost_factor: Option<f64>,
    stop_words: Option<Vec<String>>,
  ) -> Result<Query> {
    let builder = Query::more_like_this_builder(
      min_doc_frequency,
      max_doc_frequency,
      min_term_frequency,
      max_query_terms,
      min_word_length,
      max_word_length,
      boost_factor,
      stop_words,
    );
    Ok(boxed(
      builder.with_document(tv::DocAddress::from(&doc_address)),
    ))
  }

  /// Construct a Tantivy's MoreLikeThisQuery from caller-provided field values.
  ///
  /// @param schema - Schema of the target index.
  /// @param documentFields - An object mapping field names to their value(s).
  #[napi(factory)]
  #[allow(clippy::too_many_arguments)]
  pub fn more_like_this_document_fields_query(
    schema: &Schema,
    document_fields: Object,
    min_doc_frequency: Option<f64>,
    max_doc_frequency: Option<f64>,
    min_term_frequency: Option<u32>,
    max_query_terms: Option<u32>,
    min_word_length: Option<u32>,
    max_word_length: Option<u32>,
    boost_factor: Option<f64>,
    stop_words: Option<Vec<String>>,
  ) -> Result<Query> {
    // Tantivy's provided-fields MLT path operates on field ids, so the
    // binding must resolve caller-provided field values against the target
    // schema before constructing the query object.
    let doc_fields = Document::field_values_from_dict(&document_fields, schema)?
      .into_iter()
      .map(|(field_name, values)| Ok((get_field(&schema.inner, &field_name)?, values)))
      .collect::<Result<Vec<_>>>()?;

    let builder = Query::more_like_this_builder(
      min_doc_frequency.or(Some(5.0)),
      max_doc_frequency,
      min_term_frequency.or(Some(2)),
      max_query_terms.or(Some(25)),
      min_word_length,
      max_word_length,
      boost_factor.or(Some(1.0)),
      stop_words,
    );
    Ok(boxed(builder.with_document_fields(doc_fields)))
  }

  /// Construct a Tantivy's ConstScoreQuery
  #[napi(factory)]
  pub fn const_score_query(query: &Query, score: f64) -> Result<Query> {
    Ok(boxed(tv::query::ConstScoreQuery::new(
      query.inner.box_clone(),
      score as f32,
    )))
  }

  /// Construct a range query over a numeric, date or IP address field.
  ///
  /// Pass `null` for `lowerBound` or `upperBound` to leave that side
  /// unbounded. Both bounds cannot be null; use `Query.allQuery()` to match
  /// all documents. Setting `includeLower` or `includeUpper` to false while
  /// the corresponding bound is null is an error — unbounded sides are always
  /// inclusive by definition.
  ///
  /// @param schema - Schema of the target index.
  /// @param fieldName - Field name to be searched.
  /// @param fieldType - Type of the field.
  /// @param lowerBound - Lower bound value, or null for unbounded.
  /// @param upperBound - Upper bound value, or null for unbounded.
  /// @param includeLower - Whether the lower bound is inclusive. Defaults to true.
  /// @param includeUpper - Whether the upper bound is inclusive. Defaults to true.
  /// @param useInvertedIndex - If true, use an inverted index range query
  ///         instead of a fast-field range query. Defaults to false.
  #[napi(factory)]
  #[allow(clippy::too_many_arguments)]
  pub fn range_query(
    schema: &Schema,
    field_name: String,
    field_type: FieldType,
    lower_bound: Option<Unknown>,
    upper_bound: Option<Unknown>,
    include_lower: Option<bool>,
    include_upper: Option<bool>,
    use_inverted_index: Option<bool>,
  ) -> Result<Query> {
    let include_lower = include_lower.unwrap_or(true);
    let include_upper = include_upper.unwrap_or(true);

    let unsupported = match field_type {
      FieldType::Text => Some("Text"),
      FieldType::Boolean => Some("Boolean"),
      FieldType::Facet => Some("Facet"),
      FieldType::Bytes => Some("Bytes"),
      FieldType::Json => Some("Json"),
      _ => None,
    };
    if let Some(name) = unsupported {
      return Err(Error::new(
        Status::InvalidArg,
        format!("{name} fields are not supported for range queries."),
      ));
    }

    // Look up the field in the schema. The given type must match the
    // field type in the schema.
    let field = get_field(&schema.inner, &field_name)?;
    let actual_field_type = schema
      .inner
      .get_field_entry(field)
      .field_type()
      .value_type();
    let given_field_type: tv::schema::Type = field_type.clone().into();

    if actual_field_type != given_field_type {
      return Err(Error::new(
        Status::InvalidArg,
        format!(
          "Field type mismatch: field '{}' is type {:?}, but got {:?}",
          field_name, actual_field_type, given_field_type
        ),
      ));
    }

    if lower_bound.is_none() && upper_bound.is_none() {
      // tv::query::RangeQuery panics if both bounds are Unbounded, so this
      // combination has to be rejected before constructing the query.
      return Err(Error::new(
        Status::InvalidArg,
        "At least one of lowerBound or upperBound must be provided. \
         To match all documents, use Query.allQuery() instead.",
      ));
    }
    if lower_bound.is_none() && !include_lower {
      return Err(Error::new(
        Status::InvalidArg,
        "includeLower=false is invalid when lowerBound is null: \
         an unbounded side is always inclusive.",
      ));
    }
    if upper_bound.is_none() && !include_upper {
      return Err(Error::new(
        Status::InvalidArg,
        "includeUpper=false is invalid when upperBound is null: \
         an unbounded side is always inclusive.",
      ));
    }

    let make_bound = |value: Option<Unknown>, include: bool| -> Result<OpsBound<tv::Term>> {
      Ok(match value {
        None => OpsBound::Unbounded,
        Some(value) => {
          let term = make_term_for_type(&schema.inner, &field_name, field_type.clone(), &value)?;
          if include {
            OpsBound::Included(term)
          } else {
            OpsBound::Excluded(term)
          }
        }
      })
    };

    let lower_bound = make_bound(lower_bound, include_lower)?;
    let upper_bound = make_bound(upper_bound, include_upper)?;

    Ok(if use_inverted_index.unwrap_or(false) {
      boxed(tv::query::InvertedIndexRangeQuery::new(
        lower_bound,
        upper_bound,
      ))
    } else {
      boxed(tv::query::RangeQuery::new(lower_bound, upper_bound))
    })
  }

  /// Explain how this query matches a given document.
  ///
  /// This method provides detailed information about how the document matched
  /// the query and how the score was calculated.
  ///
  /// @param searcher - The searcher used to perform the search.
  /// @param docAddress - The address of the document to explain.
  #[napi]
  pub fn explain(
    &self,
    searcher: &crate::searcher::Searcher,
    doc_address: DocAddress,
  ) -> Result<Explanation> {
    let explanation = self
      .inner
      .explain(&searcher.inner, tv::DocAddress::from(&doc_address))
      .map_err(to_napi_error)?;
    Ok(Explanation::new(explanation))
  }
}

/// Resolve a phrase word list to `(offset, value)` pairs. A word is either
/// the term itself — offset defaults to its index in the list — or a
/// `[offset, term]` pair placing it at an explicit position in the phrase.
fn phrase_words<'a, T>(
  words: &'a [Unknown<'a>],
  mut convert: impl FnMut(&Unknown<'a>) -> Result<T>,
) -> Result<Vec<(usize, T)>> {
  let mut out = Vec::with_capacity(words.len());
  for (idx, word) in words.iter().enumerate() {
    if word.is_array()? {
      let pair: Object = unsafe { word.cast()? };
      if pair.get_array_length()? == 2 {
        let offset: u32 = pair.get_element(0)?;
        out.push((offset as usize, convert(&pair.get_element(1)?)?));
        continue;
      }
    }
    out.push((idx, convert(word)?));
  }
  if out.is_empty() {
    return Err(Error::new(
      Status::InvalidArg,
      "words must not be empty.".to_string(),
    ));
  }
  Ok(out)
}
