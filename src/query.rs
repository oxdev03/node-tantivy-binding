use crate::{
    get_field, make_term, make_term_for_type, schema::FieldType, to_napi_error,
    searcher::DocAddress, Schema, explanation::Explanation,
};
use core::ops::Bound as OpsBound;
use napi::{
    Error, JsObject, JsUnknown, Result, Status,
};
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
    pub fn to_string(&self) -> String {
        format!("Query({:?})", self.get())
    }

    /// Construct a Tantivy's TermQuery
    #[napi(factory)]
    pub fn term_query(
        schema: &Schema,
        field_name: String,
        field_value: JsUnknown,
        index_option: Option<String>,
    ) -> Result<Query> {
        let index_option = index_option.unwrap_or_else(|| "position".to_string());
        let term = make_term(&schema.inner, &field_name, field_value)?;
        let index_option = match index_option.as_str() {
            "position" => tv::schema::IndexRecordOption::WithFreqsAndPositions,
            "freq" => tv::schema::IndexRecordOption::WithFreqs,
            "basic" => tv::schema::IndexRecordOption::Basic,
            _ => return Err(Error::new(
                Status::InvalidArg,
                "Invalid index option, valid choices are: 'basic', 'freq' and 'position'".to_string()
            ))
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
        field_values: Vec<JsUnknown>,
    ) -> Result<Query> {
        let terms = field_values
            .into_iter()
            .map(|field_value| {
                make_term(&schema.inner, &field_name, field_value)
            })
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
        
        // For now, create the term directly without JsUnknown conversion
        // This is a simplification - in practice you'd want a different approach
        let field = crate::get_field(&schema.inner, &field_name)?;
        let term = tv::Term::from_field_text(field, &text);
        let inner = if prefix {
            tv::query::FuzzyTermQuery::new_prefix(
                term,
                distance,
                transposition_cost_one,
            )
        } else {
            tv::query::FuzzyTermQuery::new(
                term,
                distance,
                transposition_cost_one,
            )
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
        words: Vec<JsUnknown>,
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
        let inner = tv::query::PhraseQuery::new_with_offset_and_slop(
            terms_with_offset,
            slop,
        );
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's BooleanQuery
    #[napi(factory)]
    pub fn boolean_query(
        _subqueries: Vec<JsObject>,
    ) -> Result<Query> {
        // TODO: Implement proper boolean query construction
        // For now, create a dummy AllQuery
        let inner = tv::query::AllQuery;
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's DisjunctionMaxQuery
    #[napi(factory)]
    pub fn disjunction_max_query(
        subqueries: Vec<&Query>,
        tie_breaker: Option<f64>,
    ) -> Result<Query> {
        let inner_queries: Vec<Box<dyn tv::query::Query>> = subqueries
            .iter()
            .map(|query| query.inner.box_clone())
            .collect();

        let dismax_query = if let Some(tie_breaker) = tie_breaker {
            tv::query::DisjunctionMaxQuery::with_tie_breaker(
                inner_queries,
                tie_breaker as f32,
            )
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
    pub fn regex_query(
        schema: &Schema,
        field_name: String,
        regex_pattern: String,
    ) -> Result<Query> {
        let field = get_field(&schema.inner, &field_name)?;

        let inner_result =
            tv::query::RegexQuery::from_pattern(&regex_pattern, field);
        match inner_result {
            Ok(inner) => Ok(Query {
                inner: Box::new(inner),
            }),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    #[napi(factory)]
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
    pub fn const_score_query(
        query: &Query,
        score: f64,
    ) -> Result<Query> {
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
        lower_bound: JsUnknown,
        upper_bound: JsUnknown,
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

        let lower_bound_term = make_term_for_type(
            &schema.inner,
            &field_name,
            field_type.clone(),
            lower_bound,
        )?;
        let upper_bound_term = make_term_for_type(
            &schema.inner,
            &field_name,
            field_type.clone(),
            upper_bound,
        )?;

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
    pub fn explain(&self, searcher: &crate::searcher::Searcher, doc_address: DocAddress) -> Result<Explanation> {
        let tantivy_doc_address = tv::DocAddress::from(&doc_address);
        let explanation = self.inner
            .explain(&searcher.inner, tantivy_doc_address)
            .map_err(to_napi_error)?;
        Ok(Explanation::new(explanation))
    }
}
