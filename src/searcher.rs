#![allow(clippy::new_ret_no_self)]

use crate::{document::Document, query::Query};
use napi::{Error, JsUnknown, Result, Status};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use tantivy as tv;
use tantivy::aggregation::AggregationCollector;
use tantivy::collector::{Count, MultiCollector, TopDocs};
use tantivy::TantivyDocument;
// Bring the trait into scope. This is required for the `to_named_doc` method.
// However, node-tantivy declares its own `Document` class, so we need to avoid
// introduce the `Document` trait into the namespace.
use tantivy::Document as _;

/// Tantivy's Searcher class
///
/// A Searcher is used to search the index given a prepared Query.
#[napi]
pub struct Searcher {
    pub(crate) inner: tv::Searcher,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
enum Fruit {
    Score(f32),
    Order(u64),
}

impl std::fmt::Debug for Fruit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fruit::Score(s) => f.write_str(&format!("{s}")),
            Fruit::Order(o) => f.write_str(&format!("{o}")),
        }
    }
}

#[napi]
#[derive(Deserialize, PartialEq, Serialize)]
/// Enum representing the direction in which something should be sorted.
pub enum Order {
    /// Ascending. Smaller values appear first.
    Asc,

    /// Descending. Larger values appear first.
    Desc,
}

impl From<Order> for tv::Order {
    fn from(order: Order) -> Self {
        match order {
            Order::Asc => tv::Order::Asc,
            Order::Desc => tv::Order::Desc,
        }
    }
}

#[napi(object)]
#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
/// Object holding a results successful search.
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    /// How many documents matched the query. Only available if `count` was set
    /// to true during the search.
    pub count: Option<u32>,
}

#[napi(object)]
#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct SearchHit {
    pub score: Option<f64>,
    pub order: Option<f64>,
    pub doc_address: DocAddress,
}

#[napi]
impl Searcher {
    /// Search the index with the given query and collect results.
    ///
    /// Args:
    ///     query (Query): The query that will be used for the search.
    ///     limit (int, optional): The maximum number of search results to
    ///         return. Defaults to 10.
    ///     count (bool, optional): Should the number of documents that match
    ///         the query be returned as well. Defaults to true.
    ///     order_by_field (Field, optional): A schema field that the results
    ///         should be ordered by. The field must be declared as a fast field
    ///         when building the schema. Note, this only works for unsigned
    ///         fields.
    ///     offset (Field, optional): The offset from which the results have
    ///         to be returned.
    ///     order (Order, optional): The order in which the results
    ///         should be sorted. If not specified, defaults to descending.
    ///
    /// Returns `SearchResult` object.
    ///
    /// Raises a ValueError if there was an error with the search.
    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        query: &Query,
        limit: Option<u32>,
        count: Option<bool>,
        order_by_field: Option<String>,
        offset: Option<u32>,
        order: Option<Order>,
    ) -> Result<SearchResult> {
        let limit = limit.unwrap_or(10) as usize;
        let count = count.unwrap_or(true);
        let offset = offset.unwrap_or(0) as usize;
        let order = order.unwrap_or(Order::Desc);

        if let Some(order_by_field) = order_by_field {
            // Order by field search
            let mut multicollector = MultiCollector::new();
            
            let count_handle = if count {
                Some(multicollector.add_collector(Count))
            } else {
                None
            };
            
            let collector = TopDocs::with_limit(limit)
                .and_offset(offset)
                .order_by_u64_field(&order_by_field, order.into());
            let top_docs_handle = multicollector.add_collector(collector);
            
            let mut multifruit = self.inner.search(&query.inner, &multicollector)
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
            
            let top_docs = top_docs_handle.extract(&mut multifruit);
            let hits: Vec<SearchHit> = top_docs
                .iter()
                .map(|(f, d)| SearchHit {
                    score: None,
                    order: Some(*f as f64),
                    doc_address: DocAddress::from(d),
                })
                .collect();
            
            let count = count_handle.map(|h| h.extract(&mut multifruit) as u32);
            Ok(SearchResult { hits, count })
        } else {
            // Score-based search
            let mut multicollector = MultiCollector::new();
            
            let count_handle = if count {
                Some(multicollector.add_collector(Count))
            } else {
                None
            };
            
            let collector = TopDocs::with_limit(limit).and_offset(offset);
            let top_docs_handle = multicollector.add_collector(collector);
            
            let mut multifruit = self.inner.search(&query.inner, &multicollector)
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
            
            let top_docs = top_docs_handle.extract(&mut multifruit);
            let hits: Vec<SearchHit> = top_docs
                .iter()
                .map(|(f, d)| SearchHit {
                    score: Some(*f as f64),
                    order: None,
                    doc_address: DocAddress::from(d),
                })
                .collect();
            
            let count = count_handle.map(|h| h.extract(&mut multifruit) as u32);
            Ok(SearchResult { hits, count })
        }
    }

    #[napi]
    pub fn aggregate(
        &self,
        query: &Query,
        agg: JsUnknown,
    ) -> Result<String> {
        // Convert the JS object to JSON string first
        let agg_str = agg.coerce_to_string()?.into_utf8()?.into_owned()?;

        let agg_collector = AggregationCollector::from_aggs(
            serde_json::from_str(&agg_str).map_err(|e| {
                Error::new(Status::InvalidArg, format!("Invalid aggregation JSON: {}", e))
            })?,
            Default::default(),
        );
        
        let agg_res = self
            .inner
            .search(&query.inner, &agg_collector)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        let result_str = serde_json::to_string(&agg_res)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(result_str)
    }

    /// Returns the overall number of documents in the index.
    #[napi(getter)]
    pub fn num_docs(&self) -> u32 {
        self.inner.num_docs() as u32
    }

    /// Returns the number of segments in the index.
    #[napi(getter)]
    pub fn num_segments(&self) -> u32 {
        self.inner.segment_readers().len() as u32
    }

    /// Return the overall number of documents containing
    /// the given term.
    #[napi]
    pub fn doc_freq(
        &self,
        field_name: String,
        field_value: JsUnknown,
    ) -> Result<u32> {
        // Wrap the tantivy Searcher `doc_freq` method to return a Result.
        let schema = self.inner.schema();
        let term = crate::make_term(schema, &field_name, field_value)?;
        self.inner.doc_freq(&term)
            .map(|count| count as u32)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }

    /// Fetches a document from Tantivy's store given a DocAddress.
    ///
    /// Args:
    ///     doc_address (DocAddress): The DocAddress that is associated with
    ///         the document that we wish to fetch.
    ///
    /// Returns the Document, raises ValueError if the document can't be found.
    #[napi]
    pub fn doc(&self, doc_address: DocAddress) -> Result<Document> {
        let doc: TantivyDocument =
            self.inner.doc((&doc_address).into())
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        let named_doc = doc.to_named_doc(self.inner.schema());
        Ok(crate::document::Document {
            field_values: named_doc.0,
        })
    }
}

/// DocAddress contains all the necessary information to identify a document
/// given a Searcher object.
///
/// It consists in an id identifying its segment, and its segment-local DocId.
/// The id used for the segment is actually an ordinal in the list of segment
/// hold by a Searcher.
#[napi(object)]
#[derive(
    Clone, Debug, Deserialize, PartialEq, PartialOrd, Eq, Ord, Serialize,
)]
pub struct DocAddress {
    pub segment_ord: u32,
    pub doc: u32,
}

impl From<&tv::DocAddress> for DocAddress {
    fn from(doc_address: &tv::DocAddress) -> Self {
        DocAddress {
            segment_ord: doc_address.segment_ord,
            doc: doc_address.doc_id,
        }
    }
}

impl From<&DocAddress> for tv::DocAddress {
    fn from(val: &DocAddress) -> Self {
        tv::DocAddress {
            segment_ord: val.segment_ord,
            doc_id: val.doc,
        }
    }
}
