use crate::{document::Document, query::Query, to_napi_error};
use napi::bindgen_prelude::*;
use napi::{Error, Result, Status};
use napi_derive::napi;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use tantivy as tv;
use tantivy::aggregation::AggregationCollector;
use tantivy::collector::{Collector, Count, MultiCollector, SegmentCollector, TopDocs};
use tantivy::schema::{IndexRecordOption, Type};
use tantivy::TantivyDocument;
use tantivy::{DocId, DocSet, Score, SegmentOrdinal, TERMINATED};
use tantivy_common::BitSet;
// Bring the trait into scope. This is required for the `to_named_doc` method.
// However, node-tantivy declares its own `Document` class, so we need to avoid
// introduce the `Document` trait into the namespace.
use tantivy::Document as _;

/// Returns the smallest byte string strictly greater than `prefix`.
///
/// Increments the last non-0xFF byte in place and truncates. Returns None
/// if every byte is 0xFF (caller must fall back to a manual prefix check).
fn next_prefix_bound(prefix: &[u8]) -> Option<Vec<u8>> {
  let mut bound = prefix.to_vec();
  for i in (0..bound.len()).rev() {
    if bound[i] < 0xFF {
      bound[i] += 1;
      bound.truncate(i + 1);
      return Some(bound);
    }
  }
  None
}

/// Private collector that gathers matching DocIds per segment as a BitSet.
///
/// Each segment's BitSet is sized to that segment's `max_doc()`, so total
/// memory is ~1 bit per indexed document regardless of how many docs the
/// query matches. `merge_fruits` places each segment's BitSet at index
/// `segment_ord` so `terms_with_prefix` can look up visibility by segment
/// index. Slots for segments that produced no fruit remain `None`.
struct PerSegmentBitSetCollector {
  num_segments: usize,
}

struct PerSegmentBitSetSegmentCollector {
  segment_ord: u32,
  docs: BitSet,
}

impl SegmentCollector for PerSegmentBitSetSegmentCollector {
  type Fruit = (u32, BitSet);

  fn collect(&mut self, doc: DocId, _score: Score) {
    self.docs.insert(doc);
  }

  fn harvest(self) -> Self::Fruit {
    (self.segment_ord, self.docs)
  }
}

impl Collector for PerSegmentBitSetCollector {
  type Fruit = Vec<Option<BitSet>>;
  type Child = PerSegmentBitSetSegmentCollector;

  fn for_segment(
    &self,
    segment_local_id: SegmentOrdinal,
    reader: &tv::SegmentReader,
  ) -> tv::Result<Self::Child> {
    Ok(PerSegmentBitSetSegmentCollector {
      segment_ord: segment_local_id,
      docs: BitSet::with_max_value(reader.max_doc()),
    })
  }

  fn requires_scoring(&self) -> bool {
    false
  }

  fn merge_fruits(&self, segment_fruits: Vec<(u32, BitSet)>) -> tv::Result<Vec<Option<BitSet>>> {
    // None marks "no fruit for this segment". In practice tantivy calls
    // for_segment for every segment, so every slot ends up as Some(_) —
    // but a None default keeps merge_fruits robust to any future change.
    let mut result: Vec<Option<BitSet>> = (0..self.num_segments).map(|_| None).collect();
    for (seg_ord, docs) in segment_fruits {
      result[seg_ord as usize] = Some(docs);
    }
    Ok(result)
  }
}

/// Tantivy's Searcher class
///
/// A Searcher is used to search the index given a prepared Query.
#[napi]
pub struct Searcher {
  pub(crate) inner: tv::Searcher,
}

#[napi]
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
/// Object holding the results of a successful search.
pub struct SearchResult {
  pub hits: Vec<SearchHit>,
  /// How many documents matched the query. Only available if `count` was set
  /// to true during the search.
  pub count: Option<u32>,
}

#[napi(object)]
pub struct SearchHit {
  /// The relevance score. Only set when the results are not ordered by a field.
  pub score: Option<f64>,
  /// The value of the ordered field. Only set when `orderByField` was given.
  ///
  /// The variant matches the ordered field's type: numeric fields yield a
  /// number, boolean fields a boolean and text fields a string. Date fields
  /// yield milliseconds since the epoch, matching the convention used
  /// everywhere else in this binding.
  pub order: Option<Either3<f64, bool, String>>,
  pub doc_address: DocAddress,
}

#[napi(object)]
/// A term of a field paired with the number of documents containing it.
pub struct TermCount {
  pub term: String,
  pub count: u32,
}

/// Open one column per segment, map each DocAddress to its value, and wrap it
/// in the given `FastFieldValue` variant. Used by `fastFieldValues()` to avoid
/// repeating the same iterator chain for each numeric type.
macro_rules! read_fast_field_column_values {
  ($readers:expr, $field:expr, $addrs:expr, $method:ident, $variant:expr) => {{
    let columns: Vec<Option<_>> = $readers
      .iter()
      .map(|reader| reader.fast_fields().$method($field).ok())
      .collect();
    Ok(
      $addrs
        .iter()
        .map(|addr| {
          columns[addr.segment_ord as usize]
            .as_ref()
            .and_then(|col| col.first(addr.doc))
            .map($variant)
        })
        .collect(),
    )
  }};
}

impl Searcher {
  /// Execute an aggregation from an already-deserialized spec. Shared by
  /// `aggregate()` and `cardinality()` so neither needs to round-trip
  /// through JSON when the spec is already a `serde_json::Value`.
  fn aggregate_value(
    &self,
    query: &Query,
    aggs: tv::aggregation::agg_req::Aggregations,
  ) -> Result<serde_json::Value> {
    let agg_collector = AggregationCollector::from_aggs(aggs, Default::default());
    let agg_res = self
      .inner
      .search(query.get(), &agg_collector)
      .map_err(to_napi_error)?;
    serde_json::to_value(agg_res).map_err(to_napi_error)
  }
}

#[napi]
impl Searcher {
  /// Search the index with the given query and collect results.
  ///
  /// @param query - The query that will be used for the search.
  /// @param limit - The maximum number of search results to
  ///         return. Defaults to 10.
  /// @param count - Should the number of documents that match
  ///         the query be returned as well. Defaults to true.
  /// @param orderByField - Name of a field that the results should be ordered
  ///         by. The field must be declared as a fast field when building the
  ///         schema. Supported field types: Text, Unsigned, Integer, Float,
  ///         Boolean and Date.
  /// @param offset - The offset from which the results have
  ///         to be returned.
  /// @param order - The order in which the results
  ///         should be sorted. If not specified, defaults to descending.
  /// @param weightByField - Name of a field that the results should be
  ///         weighted by. The field must be declared as a fast field when
  ///         building the schema. Note, this only works for Float, Integer
  ///         and Unsigned fields. The given field value is first transformed
  ///         using the formula `log2(2.0 + value)` and then multiplied with
  ///         the original score. This means that a weight field value of 0.0
  ///         results in no change to the original score. If the weight value
  ///         is negative, it is treated as 0.0.
  ///
  /// @returns SearchResult object. Each hit carries either a `score` (no
  ///         `orderByField`) or an `order` key matching the ordered field's
  ///         type; date fields yield milliseconds since the epoch.
  ///
  /// @throws if there was an error with the search.
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
    weight_by_field: Option<String>,
  ) -> Result<SearchResult> {
    let limit = limit.unwrap_or(10) as usize;
    let count = count.unwrap_or(true);
    let offset = offset.unwrap_or(0) as usize;
    let order = order.unwrap_or(Order::Desc);

    let mut multicollector = MultiCollector::new();
    let count_handle = if count {
      Some(multicollector.add_collector(Count))
    } else {
      None
    };

    let collector = TopDocs::with_limit(limit).and_offset(offset);

    let (mut multifruit, hits) = if let Some(weight_by_field) = weight_by_field {
      let collector = self.weighted_collector(collector, weight_by_field)?;
      let handle = multicollector.add_collector(collector);
      let mut fruit = self
        .inner
        .search(query.get(), &multicollector)
        .map_err(to_napi_error)?;
      let hits = handle
        .extract(&mut fruit)
        .iter()
        .map(|(f, d)| SearchHit {
          score: Some(*f as f64),
          order: None,
          doc_address: DocAddress::from(d),
        })
        .collect();
      (fruit, hits)
    } else if let Some(order_by) = order_by_field.as_deref() {
      let schema = self.inner.schema();
      let field = crate::get_field(schema, order_by)?;
      let field_type = schema.get_field_entry(field).field_type().value_type();

      // Each arm builds a differently-typed collector, so the search has to
      // run inside the macro where that type is still concrete.
      macro_rules! run_order_by_fast {
        ($t:ty, $to_key:expr) => {{
          let handle = multicollector
            .add_collector(collector.order_by_fast_field::<$t>(order_by, order.into()));
          let mut fruit = self
            .inner
            .search(query.get(), &multicollector)
            .map_err(to_napi_error)?;
          let hits = handle
            .extract(&mut fruit)
            .into_iter()
            .map(|(f, d)| SearchHit {
              score: None,
              order: f.map($to_key),
              doc_address: DocAddress::from(&d),
            })
            .collect();
          (fruit, hits)
        }};
      }

      match field_type {
        Type::U64 => run_order_by_fast!(u64, |v: u64| Either3::A(v as f64)),
        Type::I64 => run_order_by_fast!(i64, |v: i64| Either3::A(v as f64)),
        Type::F64 => run_order_by_fast!(f64, Either3::A),
        Type::Bool => run_order_by_fast!(bool, Either3::B),
        Type::Date => run_order_by_fast!(tv::DateTime, |v: tv::DateTime| {
          Either3::A(v.into_timestamp_millis() as f64)
        }),
        Type::Str => {
          let handle = multicollector
            .add_collector(collector.order_by_string_fast_field(order_by, order.into()));
          let mut fruit = self
            .inner
            .search(query.get(), &multicollector)
            .map_err(to_napi_error)?;
          let hits = handle
            .extract(&mut fruit)
            .into_iter()
            .map(|(f, d)| SearchHit {
              score: None,
              order: f.map(Either3::C),
              doc_address: DocAddress::from(&d),
            })
            .collect();
          (fruit, hits)
        }
        other => {
          return Err(Error::new(
            Status::InvalidArg,
            format!(
              "Field '{}' has type {:?}; orderByField only supports \
               Text, Unsigned, Integer, Float, Boolean and Date fast fields.",
              order_by, other
            ),
          ))
        }
      }
    } else {
      let handle = multicollector.add_collector(collector.order_by_score());
      let mut fruit = self
        .inner
        .search(query.get(), &multicollector)
        .map_err(to_napi_error)?;
      let hits = handle
        .extract(&mut fruit)
        .iter()
        .map(|(f, d)| SearchHit {
          score: Some(*f as f64),
          order: None,
          doc_address: DocAddress::from(d),
        })
        .collect();
      (fruit, hits)
    };

    let count = count_handle.map(|h| h.extract(&mut multifruit) as u32);
    Ok(SearchResult { hits, count })
  }

  /// Execute an aggregation query and return the results.
  ///
  /// @param query - The query that filters the documents to aggregate over.
  /// @param agg - The aggregation specification.
  ///
  /// @returns An object containing the aggregation results.
  #[napi]
  pub fn aggregate(&self, query: &Query, agg: serde_json::Value) -> Result<serde_json::Value> {
    let aggs = serde_json::from_value(agg)
      .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid aggregation: {e}")))?;
    self.aggregate_value(query, aggs)
  }

  /// Returns the cardinality (approximate distinct value count) of a field
  /// over the documents matching the query.
  ///
  /// @param query - The query that will be used for the search.
  /// @param fieldName - The field for which to compute the cardinality.
  #[napi]
  pub fn cardinality(&self, query: &Query, field_name: String) -> Result<f64> {
    let aggs = serde_json::from_value(serde_json::json!({
      "cardinality": { "cardinality": { "field": field_name } }
    }))
    .map_err(to_napi_error)?;

    self.aggregate_value(query, aggs)?["cardinality"]["value"]
      .as_f64()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Unexpected aggregation result"))
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
  pub fn doc_freq(&self, field_name: String, field_value: Unknown) -> Result<u32> {
    let schema = self.inner.schema();
    let term = crate::make_term(schema, &field_name, &field_value)?;
    self
      .inner
      .doc_freq(&term)
      .map(|count| count as u32)
      .map_err(to_napi_error)
  }

  /// Fetches a document from Tantivy's store given a DocAddress.
  ///
  /// @param docAddress - The DocAddress that is associated with
  ///         the document that we wish to fetch.
  ///
  /// @returns The Document, throws if the document can't be found.
  #[napi]
  pub fn doc(&self, doc_address: DocAddress) -> Result<Document> {
    let doc: TantivyDocument = self
      .inner
      .doc((&doc_address).into())
      .map_err(to_napi_error)?;
    let named_doc = doc.to_named_doc(self.inner.schema());
    Ok(crate::document::Document {
      field_values: named_doc.0,
    })
  }

  /// Read a numeric fast field for a batch of DocAddresses without fetching
  /// stored documents.
  ///
  /// Fast fields are column-oriented and support O(1) random access by
  /// segment-local DocId. Use this instead of `doc().toDict()[field]` when
  /// you only need a single numeric field for many documents.
  ///
  /// @param fieldName - Name of a u64, i64, f64 or boolean field declared as fast.
  /// @param docAddresses - The addresses to read (e.g. from `search().hits`).
  ///
  /// @returns The values in the same order as `docAddresses`. `null` is
  ///         returned for any address where the column is absent (e.g. a
  ///         segment written before the field was added to the schema).
  ///
  /// @throws if the field does not exist, is not a fast field, or has an
  ///         unsupported type.
  #[napi]
  pub fn fast_field_values(
    &self,
    field_name: String,
    doc_addresses: Vec<DocAddress>,
  ) -> Result<Vec<Option<Either<f64, bool>>>> {
    let schema = self.inner.schema();
    let field = schema
      .get_field(&field_name)
      .map_err(|_| Error::new(Status::InvalidArg, format!("Unknown field: '{field_name}'")))?;
    let field_entry = schema.get_field_entry(field);
    if !field_entry.is_fast() {
      return Err(Error::new(
        Status::InvalidArg,
        format!("Field '{field_name}' is not a fast field."),
      ));
    }

    let segment_readers = self.inner.segment_readers();
    let num_segments = segment_readers.len();

    // Validate all segment_ords before reading so we don't produce a
    // partial result on error.
    for doc_address in &doc_addresses {
      if doc_address.segment_ord as usize >= num_segments {
        return Err(Error::new(
          Status::InvalidArg,
          format!("Invalid segmentOrd: {}", doc_address.segment_ord),
        ));
      }
    }

    // Pre-open one Column per segment so it is not reopened per document.
    // Column::first() returns Option<T>, so no sentinel value is needed.
    let field_name = field_name.as_str();
    match field_entry.field_type().value_type() {
      Type::U64 => {
        read_fast_field_column_values!(segment_readers, field_name, doc_addresses, u64, |v: u64| {
          Either::A(v as f64)
        })
      }
      Type::I64 => {
        read_fast_field_column_values!(segment_readers, field_name, doc_addresses, i64, |v: i64| {
          Either::A(v as f64)
        })
      }
      Type::F64 => {
        read_fast_field_column_values!(segment_readers, field_name, doc_addresses, f64, Either::A)
      }
      Type::Bool => {
        read_fast_field_column_values!(segment_readers, field_name, doc_addresses, bool, Either::B)
      }
      _ => Err(Error::new(
        Status::InvalidArg,
        format!(
          "Field '{field_name}' has unsupported type for fast field access. \
           Only u64, i64, f64 and boolean fast fields are supported."
        ),
      )),
    }
  }

  /// Walk the term dictionary for `fieldName` and return all terms that
  /// begin with `prefix`, together with their document frequencies.
  ///
  /// @param fieldName - Name of an indexed text field in the schema.
  /// @param prefix - Only terms beginning with this string are returned.
  ///         An empty string returns all terms in the field.
  /// @param filterQuery - When provided, each term's count reflects only
  ///         documents matched by the query (e.g. for permission filtering).
  ///         Counts are still summed across segments.
  /// @param limit - If given, only the top-`limit` entries (by count) are returned.
  ///
  /// @returns `[{ term, count }, ...]` sorted by count descending, then
  ///         alphabetically. Terms present in multiple segments have their
  ///         counts summed.
  ///
  /// @throws if the field does not exist or is not a text field.
  #[napi]
  pub fn terms_with_prefix(
    &self,
    field_name: String,
    prefix: String,
    filter_query: Option<&Query>,
    limit: Option<u32>,
  ) -> Result<Vec<TermCount>> {
    let schema = self.inner.schema();
    let field = crate::get_field(schema, &field_name)?;
    if !matches!(
      schema.get_field_entry(field).field_type().value_type(),
      Type::Str
    ) {
      return Err(Error::new(
        Status::InvalidArg,
        format!("Field '{field_name}' is not an indexed text field."),
      ));
    }

    let prefix_bytes = prefix.as_bytes();
    let upper_bound = next_prefix_bound(prefix_bytes);
    // When every byte of prefix is 0xFF no FST upper bound can be expressed;
    // the inner loop falls back to a manual starts_with check.
    let open_ended = upper_bound.is_none() && !prefix_bytes.is_empty();
    let num_segments = self.inner.segment_readers().len();

    let filter_sets: Option<Vec<Option<BitSet>>> = filter_query
      .map(|fq| {
        self
          .inner
          .search(fq.get(), &PerSegmentBitSetCollector { num_segments })
          .map_err(to_napi_error)
      })
      .transpose()?;

    if let Some(ref sets) = filter_sets {
      if sets
        .iter()
        .all(|s| s.as_ref().is_none_or(|bs| bs.len() == 0))
      {
        return Ok(vec![]);
      }
    }

    let mut counts: HashMap<String, u32> = HashMap::new();

    for (seg_ord, segment_reader) in self.inner.segment_readers().iter().enumerate() {
      // Resolve this segment's filter once per segment, not per term.
      // - None outer  → no filter at all; use term doc_freq below.
      // - Some(None)  → segment produced no fruit; skip it entirely.
      // - Some(Some(bs)) with len() == 0 → no docs match here; skip.
      // - Some(Some(bs)) with len() > 0  → intersect postings against bs.
      let segment_filter: Option<&BitSet> = match &filter_sets {
        None => None,
        Some(sets) => match sets[seg_ord].as_ref() {
          Some(bs) if bs.len() > 0 => Some(bs),
          _ => continue,
        },
      };

      let inv_index = segment_reader
        .inverted_index(field)
        .map_err(to_napi_error)?;

      let mut stream = {
        let mut builder = inv_index.terms().range().ge(prefix_bytes);
        if let Some(ref ub) = upper_bound {
          builder = builder.lt(ub.as_slice());
        }
        builder.into_stream().map_err(to_napi_error)?
      };

      while stream.advance() {
        let key = stream.key();
        if open_ended && !key.starts_with(prefix_bytes) {
          break;
        }
        let Ok(term_str) = std::str::from_utf8(key) else {
          continue;
        };

        let count = match segment_filter {
          None => stream.value().doc_freq,
          Some(filter_set) => {
            let mut postings = inv_index
              .read_postings_from_terminfo(stream.value(), IndexRecordOption::Basic)
              .map_err(to_napi_error)?;
            let mut c = 0u32;
            // SegmentPostings initialises at doc 0; read doc() before the
            // first advance().
            loop {
              let doc = postings.doc();
              if doc == TERMINATED {
                break;
              }
              if filter_set.contains(doc) {
                c += 1;
              }
              postings.advance();
            }
            c
          }
        };

        if count > 0 {
          *counts.entry(term_str.to_owned()).or_insert(0) += count;
        }
      }
    }

    let pairs: Vec<(String, u32)> = match limit.map(|l| l as usize) {
      None => {
        let mut pairs: Vec<(String, u32)> = counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        pairs
      }
      Some(0) => Vec::new(),
      Some(n) => {
        // Bounded min-heap of size n. The key (count, Reverse(term)) is
        // constructed so "larger" means "more deserving" — higher count, or
        // on ties, lexicographically smaller term. BinaryHeap is a max-heap,
        // so wrapping in an outer Reverse flips it to a min-heap whose peek()
        // is the worst currently kept entry — the candidate for eviction when
        // a new term outranks it.
        let mut heap: BinaryHeap<Reverse<(u32, Reverse<String>)>> = BinaryHeap::with_capacity(n);
        for (term, count) in counts {
          let key = (count, Reverse(term));
          if heap.len() < n {
            heap.push(Reverse(key));
          } else if heap.peek().is_some_and(|Reverse(worst)| &key > worst) {
            heap.pop();
            heap.push(Reverse(key));
          }
        }
        let mut top: Vec<(u32, Reverse<String>)> = heap.into_iter().map(|Reverse(k)| k).collect();
        // Heap order is unspecified; sort the survivors descending (largest
        // key first) for the documented output order.
        top.sort_by(|a, b| b.cmp(a));
        top
          .into_iter()
          .map(|(count, Reverse(term))| (term, count))
          .collect()
      }
    };

    Ok(
      pairs
        .into_iter()
        .map(|(term, count)| TermCount { term, count })
        .collect(),
    )
  }

  /// Convert the searcher to a string representation
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    format!(
      "Searcher(numDocs={}, numSegments={})",
      self.inner.num_docs(),
      self.inner.segment_readers().len()
    )
  }
}

impl Searcher {
  /// Wrap `collector` so each score is multiplied by `log2(2 + fieldValue)`.
  fn weighted_collector(
    &self,
    collector: TopDocs,
    weight_by_field: String,
  ) -> Result<impl tv::collector::Collector<Fruit = Vec<(tv::Score, tv::DocAddress)>>> {
    let schema = self.inner.schema();
    let field = crate::get_field(schema, &weight_by_field)?;
    let field_entry = schema.get_field_entry(field);
    let field_type = field_entry.field_type().value_type();

    if !field_entry.is_fast() {
      return Err(Error::new(
        Status::InvalidArg,
        format!(
          "Field '{weight_by_field}' is not a fast field. The field must be declared as fast in the schema."
        ),
      ));
    }

    if !matches!(field_type, Type::F64 | Type::I64 | Type::U64) {
      return Err(Error::new(
        Status::InvalidArg,
        format!(
          "Unsupported field type for weighting: {field_type:?}. Only f64, i64 and u64 fast fields are supported."
        ),
      ));
    }

    Ok(
      collector.tweak_score(move |segment_reader: &tv::SegmentReader| {
        // All three readers are created upfront even though only one matches
        // the field type: a Rust closure has a single concrete type, so the
        // arms cannot return different closures, and Box<dyn Fn> would add a
        // heap allocation per segment plus virtual dispatch per document.
        let f64_reader = segment_reader
          .fast_fields()
          .f64(&weight_by_field)
          .ok()
          .map(|r| r.first_or_default_col(0.0));
        let i64_reader = segment_reader
          .fast_fields()
          .i64(&weight_by_field)
          .ok()
          .map(|r| r.first_or_default_col(0));
        let u64_reader = segment_reader
          .fast_fields()
          .u64(&weight_by_field)
          .ok()
          .map(|r| r.first_or_default_col(0));

        move |doc: tv::DocId, original_score: tv::Score| {
          // map_or(0.0, ...) rather than unwrap(): segments created before a
          // schema change may lack this fast field. A default of 0.0 is
          // neutral, since log2(2.0 + 0.0) == 1.0.
          let value: f64 = match field_type {
            Type::F64 => f64_reader.as_ref().map_or(0.0, |r| r.get_val(doc)),
            Type::I64 => i64_reader.as_ref().map_or(0.0, |r| r.get_val(doc) as f64),
            Type::U64 => u64_reader.as_ref().map_or(0.0, |r| r.get_val(doc) as f64),
            _ => unreachable!("field type validated above"),
          };
          let value = value.max(0.0); // Negative values are not allowed
          ((2f64 + value) as tv::Score).log2() * original_score
        }
      }),
    )
  }
}

/// DocAddress contains all the necessary information to identify a document
/// given a Searcher object.
///
/// It consists in an id identifying its segment, and its segment-local DocId.
/// The id used for the segment is actually an ordinal in the list of segment
/// hold by a Searcher.
#[napi(object)]
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
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
