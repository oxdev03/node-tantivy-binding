use crate::{
  explanation::Explanation, get_field, make_term, make_term_for_type, schema::FieldType,
  searcher::DocAddress, to_napi_error, Schema,
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
    let index_option = index_option.unwrap_or_else(|| "position".to_string());
    let term = make_term(&schema.inner, &field_name, field_value)?;
    let index_option = match index_option.as_str() {
      "position" => tv::schema::IndexRecordOption::WithFreqsAndPositions,
      "freq" => tv::schema::IndexRecordOption::WithFreqs,
      "basic" => tv::schema::IndexRecordOption::Basic,
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          "Invalid index option, valid choices are: 'basic', 'freq' and 'position'".to_string(),
        ))
      }
    };
    let inner = tv::query::TermQuery::new(term, index_option);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's TermSetQuery
  #[napi(factory)]
  pub fn term_set_query(
    schema: &Schema,
    field_name: String,
    field_values: Vec<Unknown>,
  ) -> Result<Query> {
    let terms = field_values
      .into_iter()
      .map(|field_value| make_term(&schema.inner, &field_name, field_value))
      .collect::<Result<Vec<_>>>()?;
    let inner = tv::query::TermSetQuery::new(terms);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's AllQuery
  #[napi(factory)]
  pub fn all_query() -> Result<Query> {
    let inner = tv::query::AllQuery {};
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's EmptyQuery
  ///
  /// A query that matches no documents. Useful as a placeholder or default.
  #[napi(factory)]
  pub fn empty_query() -> Result<Query> {
    let inner = tv::query::EmptyQuery {};
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's ExistsQuery
  ///
  /// Matches all documents that have at least one non-null value in the given field.
  /// Useful for filtering documents that have a specific field populated.
  ///
  /// # Arguments
  ///
  /// * `schema` - Schema of the target index.
  /// * `field_name` - Field name to check for existence.
  #[napi(factory)]
  pub fn exists_query(schema: &Schema, field_name: String) -> Result<Query> {
    // Validate the field exists in the schema
    let _field = get_field(&schema.inner, &field_name)?;
    let inner = tv::query::ExistsQuery::new(field_name, false);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's FuzzyTermQuery
  ///
  /// # Arguments
  ///
  /// * `schema` - Schema of the target index.
  /// * `field_name` - Field name to be searched.
  /// * `text` - String representation of the query term.
  /// * `distance` - (Optional) Edit distance you are going to alow. When not specified, the default is 1.
  /// * `transposition_cost_one` - (Optional) If true, a transposition (swapping) cost will be 1; otherwise it will be 2. When not specified, the default is true.
  /// * `prefix` - (Optional) If true, prefix levenshtein distance is applied. When not specified, the default is false.
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
    let prefix = prefix.unwrap_or(false);

    let field = crate::get_field(&schema.inner, &field_name)?;
    let term = tv::Term::from_field_text(field, &text);
    let inner = if prefix {
      tv::query::FuzzyTermQuery::new_prefix(term, distance, transposition_cost_one)
    } else {
      tv::query::FuzzyTermQuery::new(term, distance, transposition_cost_one)
    };
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's PhraseQuery with custom offsets and slop
  ///
  /// # Arguments
  ///
  /// * `schema` - Schema of the target index.
  /// * `field_name` - Field name to be searched.
  /// * `words` - Word list that constructs the phrase. A word can be a term text or a pair of term text and its offset in the phrase.
  /// * `slop` - (Optional) The number of gaps permitted between the words in the query phrase. Default is 0.
  #[napi(factory)]
  pub fn phrase_query(
    schema: &Schema,
    field_name: String,
    words: Vec<Unknown>,
    slop: Option<u32>,
  ) -> Result<Query> {
    let slop = slop.unwrap_or(0);
    let mut terms_with_offset = Vec::with_capacity(words.len());
    for (idx, word) in words.into_iter().enumerate() {
      // For now, we'll use the list index as the offset since napi-rs
      // doesn't have a direct equivalent to PyO3's tuple extraction
      let term = make_term(&schema.inner, &field_name, word)?;
      terms_with_offset.push((idx, term));
    }
    if terms_with_offset.is_empty() {
      return Err(Error::new(
        Status::InvalidArg,
        "words must not be empty.".to_string(),
      ));
    }
    let inner = tv::query::PhraseQuery::new_with_offset_and_slop(terms_with_offset, slop);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's BooleanQuery
  #[napi(factory)]
  pub fn boolean_query(subqueries: Vec<Object>) -> Result<Query> {
    let mut dyn_subqueries = Vec::new();

    for subquery_obj in subqueries {
      // Extract the occur and query from the object
      // Expected format: { occur: Occur, query: Query }
      let occur_value: Unknown = subquery_obj
        .get("occur")?
        .ok_or_else(|| Error::new(Status::InvalidArg, "Missing 'occur' field in subquery"))?;
      let query_value: &Query = subquery_obj
        .get("query")?
        .ok_or_else(|| Error::new(Status::InvalidArg, "Missing 'query' field in subquery"))?;

      // Convert the occur value to our Occur enum
      let occur_num: u32 = occur_value.coerce_to_number()?.get_uint32()?;
      let occur = match occur_num {
        0 => tv::query::Occur::Must,
        1 => tv::query::Occur::Should,
        2 => tv::query::Occur::MustNot,
        _ => {
          return Err(Error::new(
            Status::InvalidArg,
            "Invalid occur value, must be 0 (Must), 1 (Should), or 2 (MustNot)",
          ))
        }
      };

      dyn_subqueries.push((occur, query_value.inner.box_clone()));
    }

    let inner = tv::query::BooleanQuery::from(dyn_subqueries);

    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's DisjunctionMaxQuery
  #[napi(factory)]
  pub fn disjunction_max_query(subqueries: Vec<&Query>, tie_breaker: Option<f64>) -> Result<Query> {
    let inner_queries: Vec<Box<dyn tv::query::Query>> = subqueries
      .iter()
      .map(|query| query.inner.box_clone())
      .collect();

    let dismax_query = if let Some(tie_breaker) = tie_breaker {
      tv::query::DisjunctionMaxQuery::with_tie_breaker(inner_queries, tie_breaker as f32)
    } else {
      tv::query::DisjunctionMaxQuery::new(inner_queries)
    };

    Ok(Query {
      inner: Box::new(dismax_query),
    })
  }

  /// Construct a Tantivy's BoostQuery
  #[napi(factory)]
  pub fn boost_query(query: &Query, boost: f64) -> Result<Query> {
    let inner = tv::query::BoostQuery::new(query.inner.box_clone(), boost as f32);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's RegexQuery
  #[napi(factory)]
  pub fn regex_query(schema: &Schema, field_name: String, regex_pattern: String) -> Result<Query> {
    let field = get_field(&schema.inner, &field_name)?;

    let inner_result = tv::query::RegexQuery::from_pattern(&regex_pattern, field);
    match inner_result {
      Ok(inner) => Ok(Query {
        inner: Box::new(inner),
      }),
      Err(e) => Err(to_napi_error(e)),
    }
  }

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
    if let Some(stop_words) = stop_words {
      builder = builder.with_stop_words(stop_words);
    }

    let inner = builder.with_document(tv::DocAddress::from(&doc_address));
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's ConstScoreQuery
  #[napi(factory)]
  pub fn const_score_query(query: &Query, score: f64) -> Result<Query> {
    let inner = tv::query::ConstScoreQuery::new(query.inner.box_clone(), score as f32);
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  #[napi(factory)]
  pub fn range_query(
    schema: &Schema,
    field_name: String,
    field_type: FieldType,
    lower_bound: Unknown,
    upper_bound: Unknown,
    include_lower: Option<bool>,
    include_upper: Option<bool>,
  ) -> Result<Query> {
    let include_lower = include_lower.unwrap_or(true);
    let include_upper = include_upper.unwrap_or(true);

    match field_type {
      FieldType::Str => {
        return Err(Error::new(
          Status::InvalidArg,
          "Text fields are not supported for range queries.".to_string(),
        ))
      }
      FieldType::Bool => {
        return Err(Error::new(
          Status::InvalidArg,
          "Boolean fields are not supported for range queries.".to_string(),
        ))
      }
      FieldType::Facet => {
        return Err(Error::new(
          Status::InvalidArg,
          "Facet fields are not supported for range queries.".to_string(),
        ))
      }
      FieldType::Bytes => {
        return Err(Error::new(
          Status::InvalidArg,
          "Bytes fields are not supported for range queries.".to_string(),
        ))
      }
      FieldType::JsonObject => {
        return Err(Error::new(
          Status::InvalidArg,
          "Json fields are not supported for range queries.".to_string(),
        ))
      }
      _ => {}
    }

    // Look up the field in the schema. The given type must match the
    // field type in the schema.
    let field = get_field(&schema.inner, &field_name)?;
    let actual_field_entry = schema.inner.get_field_entry(field);
    let actual_field_type = actual_field_entry.field_type().value_type(); // Convert tv::schema::FieldType to local FieldType
    let given_field_type: tv::schema::Type = field_type.clone().into(); // Convert local FieldType to tv::schema::FieldType

    if actual_field_type != given_field_type {
      return Err(Error::new(
        Status::InvalidArg,
        format!(
          "Field type mismatch: field '{}' is type {:?}, but got {:?}",
          field_name, actual_field_type, given_field_type
        ),
      ));
    }

    let lower_bound_term =
      make_term_for_type(&schema.inner, &field_name, field_type.clone(), lower_bound)?;
    let upper_bound_term =
      make_term_for_type(&schema.inner, &field_name, field_type.clone(), upper_bound)?;

    let lower_bound = if include_lower {
      OpsBound::Included(lower_bound_term)
    } else {
      OpsBound::Excluded(lower_bound_term)
    };

    let upper_bound = if include_upper {
      OpsBound::Included(upper_bound_term)
    } else {
      OpsBound::Excluded(upper_bound_term)
    };

    let inner = tv::query::RangeQuery::new(lower_bound, upper_bound);

    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's PhrasePrefixQuery
  ///
  /// Matches a specific sequence of words followed by a term of which only a prefix is known.
  /// Requires positions to be indexed on the target field. At least two terms are required.
  ///
  /// # Arguments
  ///
  /// * `schema` - Schema of the target index.
  /// * `field_name` - Field name to be searched.
  /// * `words` - Word list that constructs the phrase. The last word is treated as a prefix.
  /// * `max_expansions` - (Optional) Maximum number of terms the prefix can expand to. Default is 50.
  #[napi(factory)]
  pub fn phrase_prefix_query(
    schema: &Schema,
    field_name: String,
    words: Vec<String>,
    max_expansions: Option<u32>,
  ) -> Result<Query> {
    if words.len() < 2 {
      return Err(Error::new(
        Status::InvalidArg,
        "PhrasePrefixQuery requires at least two terms.".to_string(),
      ));
    }
    let field = get_field(&schema.inner, &field_name)?;
    let terms: Vec<tv::Term> = words
      .iter()
      .map(|w| tv::Term::from_field_text(field, w))
      .collect();
    let mut inner = tv::query::PhrasePrefixQuery::new(terms);
    if let Some(max_exp) = max_expansions {
      inner.set_max_expansions(max_exp);
    }
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Construct a Tantivy's RegexPhraseQuery
  ///
  /// Matches a specific sequence of regex patterns in positional order, with optional slop.
  /// Each pattern can match multiple indexed terms via regex expansion.
  ///
  /// # Arguments
  ///
  /// * `schema` - Schema of the target index.
  /// * `field_name` - Field name to be searched.
  /// * `patterns` - List of regex patterns forming the phrase. Each pattern can be a string
  ///   (offset = index) or a [offset, pattern] pair for custom positioning.
  /// * `slop` - (Optional) Number of gaps permitted between matched terms. Default is 0.
  /// * `max_expansions` - (Optional) Maximum number of terms each regex can expand to.
  #[napi(factory)]
  pub fn regex_phrase_query(
    schema: &Schema,
    field_name: String,
    patterns: Vec<String>,
    slop: Option<u32>,
    max_expansions: Option<u32>,
  ) -> Result<Query> {
    if patterns.is_empty() {
      return Err(Error::new(
        Status::InvalidArg,
        "patterns must not be empty.".to_string(),
      ));
    }
    let field = get_field(&schema.inner, &field_name)?;
    let terms_with_offset: Vec<(usize, String)> = patterns.into_iter().enumerate().collect();
    let mut inner = tv::query::RegexPhraseQuery::new_with_offset(field, terms_with_offset);
    if let Some(s) = slop {
      inner.set_slop(s);
    }
    if let Some(max_exp) = max_expansions {
      inner.set_max_expansions(max_exp);
    }
    Ok(Query {
      inner: Box::new(inner),
    })
  }

  /// Explain how this query matches a given document.
  ///
  /// This method provides detailed information about how the document matched the query
  /// and how the score was calculated.
  ///
  /// # Arguments
  /// * `searcher` - The searcher used to perform the search
  /// * `doc_address` - The address of the document to explain
  ///
  /// # Returns
  /// * `Explanation` - An object containing detailed scoring information
  #[napi]
  pub fn explain(
    &self,
    searcher: &crate::searcher::Searcher,
    doc_address: DocAddress,
  ) -> Result<Explanation> {
    let tantivy_doc_address = tv::DocAddress::from(&doc_address);
    let explanation = self
      .inner
      .explain(&searcher.inner, tantivy_doc_address)
      .map_err(to_napi_error)?;
    Ok(Explanation::new(explanation))
  }
}
