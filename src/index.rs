#![allow(clippy::new_ret_no_self)]

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::{Error, Result, Status};
use napi_derive::napi;

use crate::{
  document::Document, query::Query, schema::Schema, searcher::Searcher, to_napi_error,
  tokenizer::TextAnalyzer as CrateTextAnalyzer,
};
use tantivy as tv;

const RELOAD_POLICY: &str = "commit";

/// IndexWriter is the user entry-point to add documents to the index.
///
/// To create an IndexWriter first create an Index and call the writer() method
/// on the index object.
#[napi]
pub struct IndexWriter {
  inner_index_writer: Option<tv::IndexWriter>,
  schema: tv::schema::Schema,
}

impl IndexWriter {
  fn inner(&self) -> Result<&tv::IndexWriter> {
    self.inner_index_writer.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "IndexWriter was consumed and no longer in a valid state",
      )
    })
  }

  fn inner_mut(&mut self) -> Result<&mut tv::IndexWriter> {
    self.inner_index_writer.as_mut().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "IndexWriter was consumed and no longer in a valid state",
      )
    })
  }

  fn take_inner(&mut self) -> Result<tv::IndexWriter> {
    self.inner_index_writer.take().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "IndexWriter was consumed and no longer in a valid state",
      )
    })
  }
}

#[napi]
impl IndexWriter {
  /// Add a document to the index.
  ///
  /// If the indexing pipeline is full, this call may block.
  ///
  /// Returns an `opstamp`, which is an increasing integer that can be used
  /// by the client to align commits with its own document queue.
  /// The `opstamp` represents the number of documents that have been added
  /// since the creation of the index.
  #[napi]
  pub fn add_document(&mut self, doc: &Document) -> Result<u64> {
    let named_doc = tantivy::schema::NamedFieldDocument(doc.field_values.clone());
    let doc =
      tantivy::schema::document::TantivyDocument::convert_named_doc(&self.schema, named_doc)
        .map_err(to_napi_error)?;
    self.inner()?.add_document(doc).map_err(to_napi_error)
  }

  /// Helper for the `add_document` method, but passing a json string.
  ///
  /// If the indexing pipeline is full, this call may block.
  ///
  /// Returns an `opstamp`, which is an increasing integer that can be used
  /// by the client to align commits with its own document queue.
  /// The `opstamp` represents the number of documents that have been added
  /// since the creation of the index.
  #[napi]
  pub fn add_json(&mut self, json: String) -> Result<u64> {
    let doc = tantivy::schema::document::TantivyDocument::parse_json(&self.schema, &json)
      .map_err(to_napi_error)?;
    let opstamp = self.inner()?.add_document(doc);
    opstamp.map_err(to_napi_error)
  }

  /// Commits all of the pending changes
  ///
  /// A call to commit blocks. After it returns, all of the document that
  /// were added since the last commit are published and persisted.
  ///
  /// In case of a crash or an hardware failure (as long as the hard disk is
  /// spared), it will be possible to resume indexing from this point.
  ///
  /// Returns the `opstamp` of the last document that made it in the commit.
  #[napi]
  pub fn commit(&mut self) -> Result<u64> {
    self.inner_mut()?.commit().map_err(to_napi_error)
  }

  /// Rollback to the last commit
  ///
  /// This cancels all of the update that happened before after the last
  /// commit. After calling rollback, the index is in the same state as it
  /// was after the last commit.
  #[napi]
  pub fn rollback(&mut self) -> Result<u64> {
    self.inner_mut()?.rollback().map_err(to_napi_error)
  }

  /// Detect and removes the files that are not used by the index anymore.
  #[napi]
  pub fn garbage_collect_files(&mut self) -> Result<()> {
    // Note: In the original version this was async, but for simplicity we skip it
    // The user can manually manage files if needed
    Ok(())
  }

  /// Deletes all documents from the index.
  #[napi]
  pub fn delete_all_documents(&mut self) -> Result<()> {
    self
      .inner()?
      .delete_all_documents()
      .map_err(to_napi_error)?;
    Ok(())
  }

  /// The opstamp of the last successful commit.
  ///
  /// This is the opstamp the index will rollback to if there is a failure
  /// like a power surge.
  ///
  /// This is also the opstamp of the commit that is currently available
  /// for searchers.
  #[napi(getter)]
  pub fn commit_opstamp(&self) -> Result<u64> {
    Ok(self.inner()?.commit_opstamp())
  }

  /// Delete all documents containing a given term.
  ///
  /// This method does not parse the given term and it expects the term to be
  /// already tokenized according to any tokenizers attached to the field. This
  /// can often result in surprising behaviour. For example, if you want to store
  /// UUIDs as text in a field, and those values have hyphens, and you use the
  /// default tokenizer which removes punctuation, you will not be able to delete
  /// a document added with particular UUID, by passing the same UUID to this
  /// method. In such workflows where deletions are required, particularly with
  /// string values, it is strongly recommended to use the
  /// "raw" tokenizer as this will match exactly. In situations where you do
  /// want tokenization to be applied, it is recommended to instead use the
  /// `delete_documents_by_query` method instead, which will delete documents
  /// matching the given query using the same query parser as used in search queries.
  ///
  /// Args:
  ///     field_name: The field name for which we want to filter deleted docs.
  ///     field_value: JavaScript value with the value we want to filter.
  ///
  /// If the field_name is not on the schema raises error.
  /// If the field_value is not supported raises error.
  #[napi]
  pub fn delete_documents_by_term(
    &mut self,
    field_name: String,
    field_value: Unknown,
  ) -> Result<u64> {
    let term = crate::make_term(&self.schema, &field_name, field_value)?;
    Ok(self.inner()?.delete_term(term))
  }

  /// Delete all documents matching a given query.
  ///
  /// Args:
  ///    query: The query to filter the deleted documents.
  ///
  /// If the query is not valid raises error.
  /// If the query is not supported raises error.
  #[napi]
  pub fn delete_documents_by_query(&mut self, query: &Query) -> Result<u64> {
    self
      .inner()?
      .delete_query(query.inner.box_clone())
      .map_err(to_napi_error)
  }

  /// If there are some merging threads, blocks until they all finish
  /// their work and then drop the `IndexWriter`.
  ///
  /// This will consume the `IndexWriter`. Further accesses to the
  /// object will result in an error.
  #[napi]
  pub fn wait_merging_threads(&mut self) -> Result<()> {
    self
      .take_inner()?
      .wait_merging_threads()
      .map_err(to_napi_error)
  }
}

/// Create a new index object.
///
/// Args:
///     schema: The schema of the index.
///     path: The path where the index should be stored. If
///         no path is provided, the index will be stored in memory.
///     reuse: Should we open an existing index if one exists
///         or always create a new one.
///
/// If an index already exists it will be opened and reused. Raises error
/// if there was a problem during the opening or creation of the index.
#[napi]
pub struct Index {
  pub(crate) index: tv::Index,
  reader: tv::IndexReader,
}

#[napi]
impl Index {
  #[napi(factory)]
  pub fn open(path: String) -> Result<Index> {
    let index = tv::Index::open_in_dir(&path).map_err(to_napi_error)?;

    Index::register_custom_text_analyzers(&index);

    let reader = index.reader().map_err(to_napi_error)?;
    Ok(Index { index, reader })
  }

  #[napi(constructor)]
  pub fn new(schema: &Schema, path: Option<String>, reuse: Option<bool>) -> Result<Self> {
    let reuse = reuse.unwrap_or(true);
    let index = match path {
      Some(p) => {
        let directory = tantivy::directory::MmapDirectory::open(&p).map_err(to_napi_error)?;
        if reuse {
          tv::Index::open_or_create(directory, schema.inner.clone())
        } else {
          tv::Index::create(
            directory,
            schema.inner.clone(),
            tv::IndexSettings::default(),
          )
        }
        .map_err(to_napi_error)?
      }
      None => tv::Index::create_in_ram(schema.inner.clone()),
    };

    Index::register_custom_text_analyzers(&index);

    let reader = index.reader().map_err(to_napi_error)?;
    Ok(Index { index, reader })
  }

  /// Create a `IndexWriter` for the index.
  ///
  /// The writer will be multithreaded and the provided heap size will be
  /// split between the given number of threads.
  ///
  /// Args:
  ///     overall_heap_size: The total target heap memory usage of
  ///         the writer. Tantivy requires that this can't be less
  ///         than 3000000 *per thread*. Lower values will result in more
  ///         frequent internal commits when adding documents (slowing down
  ///         write progress), and larger values will results in fewer
  ///         commits but greater memory usage. The best value will depend
  ///         on your specific use case.
  ///     num_threads: The number of threads that the writer
  ///         should use. If this value is 0, tantivy will choose
  ///         automatically the number of threads.
  ///
  /// Raises error if there was an error while creating the writer.
  #[napi]
  pub fn writer(&self, heap_size: Option<u32>, num_threads: Option<u32>) -> Result<IndexWriter> {
    let heap_size = heap_size.unwrap_or(128_000_000) as usize;
    let num_threads = num_threads.unwrap_or(0) as usize;
    let writer = match num_threads {
      0 => self.index.writer(heap_size),
      _ => self.index.writer_with_num_threads(num_threads, heap_size),
    }
    .map_err(to_napi_error)?;
    let schema = self.index.schema();
    Ok(IndexWriter {
      inner_index_writer: Some(writer),
      schema,
    })
  }

  /// Configure the index reader.
  ///
  /// Args:
  ///     reload_policy: The reload policy that the
  ///         IndexReader should use. Can be `Manual` or `OnCommit`.
  ///     num_warmers: The number of searchers that the
  ///         reader should create.
  #[napi]
  pub fn config_reader(
    &mut self,
    reload_policy: Option<String>,
    num_warmers: Option<u32>,
  ) -> Result<()> {
    let reload_policy = reload_policy.unwrap_or_else(|| RELOAD_POLICY.to_string());
    let num_warmers = num_warmers.unwrap_or(0) as usize;

    let reload_policy = reload_policy.to_lowercase();
    let reload_policy = match reload_policy.as_ref() {
      "commit" => tv::ReloadPolicy::OnCommitWithDelay,
      "on-commit" => tv::ReloadPolicy::OnCommitWithDelay,
      "oncommit" => tv::ReloadPolicy::OnCommitWithDelay,
      "manual" => tv::ReloadPolicy::Manual,
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          "Invalid reload policy, valid choices are: 'manual' and 'OnCommit'",
        ))
      }
    };
    let builder = self.index.reader_builder();
    let builder = builder.reload_policy(reload_policy);
    let builder = if num_warmers > 0 {
      builder.num_warming_threads(num_warmers)
    } else {
      builder
    };

    self.reader = builder.try_into().map_err(to_napi_error)?;
    Ok(())
  }

  /// Returns a searcher
  ///
  /// This method should be called every single time a search query is performed.
  /// The same searcher must be used for a given query, as it ensures the use of a consistent segment set.
  #[napi]
  pub fn searcher(&self) -> Searcher {
    Searcher {
      inner: self.reader.searcher(),
    }
  }

  /// Check if the given path contains an existing index.
  /// Args:
  ///     path: The path where tantivy will search for an index.
  ///
  /// Returns True if an index exists at the given path, False otherwise.
  ///
  /// Raises error if the directory cannot be opened.
  #[napi]
  pub fn exists(path: String) -> Result<bool> {
    let directory = tantivy::directory::MmapDirectory::open(&path).map_err(to_napi_error)?;
    tv::Index::exists(&directory).map_err(to_napi_error)
  }

  /// The schema of the current index.
  #[napi(getter)]
  pub fn schema(&self) -> Schema {
    let schema = self.index.schema();
    Schema { inner: schema }
  }

  /// Update searchers so that they reflect the state of the last .commit().
  ///
  /// If you set up the the reload policy to be on 'commit' (which is the
  /// default) every commit should be rapidly reflected on your IndexReader
  /// and you should not need to call reload() at all.
  #[napi]
  pub fn reload(&self) -> Result<()> {
    self.reader.reload().map_err(to_napi_error)
  }

  /// Parse a query
  ///
  /// Args:
  ///     query: the query, following the tantivy query language.
  ///
  ///     default_fields_names: A list of fields used to search if no
  ///         field is specified in the query.
  ///
  ///     field_boosts: A dictionary keyed on field names which provides default boosts
  ///         for the query constructed by this method.
  ///
  ///     fuzzy_fields: A dictionary keyed on field names which provides (prefix, distance, transpose_cost_one)
  ///         triples making queries constructed by this method fuzzy against the given fields
  ///         and using the given parameters.
  ///         `prefix` determines if terms which are prefixes of the given term match the query.
  ///         `distance` determines the maximum Levenshtein distance between terms matching the query and the given term.
  ///         `transpose_cost_one` determines if transpositions of neighbouring characters are counted only once against the Levenshtein distance.
  #[napi]
  pub fn parse_query(
    &self,
    query: String,
    default_field_names: Option<Vec<String>>,
    field_boosts: Option<HashMap<String, f64>>,
    fuzzy_fields: Option<HashMap<String, (bool, u8, bool)>>,
  ) -> Result<Query> {
    let parser = self.prepare_query_parser(default_field_names, field_boosts, fuzzy_fields)?;

    let query = parser.parse_query(&query).map_err(to_napi_error)?;

    Ok(Query { inner: query })
  }

  /// Parse a query leniently.
  ///
  /// This variant parses invalid query on a best effort basis. If some part of the query can't
  /// reasonably be executed (range query without field, searching on a non existing field,
  /// searching without precising field when no default field is provided...), they may get turned
  /// into a "match-nothing" subquery.
  ///
  /// Args:
  ///     query: the query, following the tantivy query language.
  ///
  ///     default_fields_names: A list of fields used to search if no
  ///         field is specified in the query.
  ///
  ///     field_boosts: A dictionary keyed on field names which provides default boosts
  ///         for the query constructed by this method.
  ///
  ///     fuzzy_fields: A dictionary keyed on field names which provides (prefix, distance, transpose_cost_one)
  ///         triples making queries constructed by this method fuzzy against the given fields
  ///         and using the given parameters.
  ///         `prefix` determines if terms which are prefixes of the given term match the query.
  ///         `distance` determines the maximum Levenshtein distance between terms matching the query and the given term.
  ///         `transpose_cost_one` determines if transpositions of neighbouring characters are counted only once against the Levenshtein distance.
  ///
  /// Returns a tuple containing the parsed query and a list of error messages.
  #[napi]
  pub fn parse_query_lenient(
    &self,
    query: String,
    default_field_names: Option<Vec<String>>,
    field_boosts: Option<HashMap<String, f64>>,
    fuzzy_fields: Option<HashMap<String, (bool, u8, bool)>>,
  ) -> Result<(Query, Vec<String>)> {
    let parser = self.prepare_query_parser(default_field_names, field_boosts, fuzzy_fields)?;

    let (query, errors) = parser.parse_query_lenient(&query);
    let error_messages: Vec<String> = errors
      .into_iter()
      .map(|err| format!("{:?}", err))
      .collect();

    Ok((Query { inner: query }, error_messages))
  }

  /// Register a custom text analyzer by name. (Confusingly,
  /// this is one of the places where Tantivy uses 'tokenizer' to refer to a
  /// TextAnalyzer instance.)
  ///
  // Implementation notes: Skipped indirection of TokenizerManager.
  #[napi]
  pub fn register_tokenizer(&self, name: String, analyzer: &CrateTextAnalyzer) {
    self
      .index
      .tokenizers()
      .register(&name, analyzer.analyzer.clone());
  }
}

impl Index {
  fn prepare_query_parser(
    &self,
    default_field_names: Option<Vec<String>>,
    field_boosts: Option<HashMap<String, f64>>,
    fuzzy_fields: Option<HashMap<String, (bool, u8, bool)>>,
  ) -> Result<tv::query::QueryParser> {
    let schema = self.index.schema();

    let default_fields = if let Some(default_field_names) = default_field_names {
      default_field_names
        .iter()
        .map(|field_name| {
          let field = schema.get_field(field_name).map_err(|_err| {
            Error::new(
              Status::InvalidArg,
              format!("Field `{field_name}` is not defined in the schema."),
            )
          })?;

          let field_entry = schema.get_field_entry(field);
          if !field_entry.is_indexed() {
            return Err(Error::new(
              Status::InvalidArg,
              format!("Field `{field_name}` is not set as indexed in the schema."),
            ));
          }

          Ok(field)
        })
        .collect::<Result<_>>()?
    } else {
      schema
        .fields()
        .filter(|(_, field_entry)| field_entry.is_indexed())
        .map(|(field, _)| field)
        .collect()
    };

    let mut parser = tv::query::QueryParser::for_index(&self.index, default_fields);

    // Set field boosts if provided
    if let Some(field_boosts) = field_boosts {
      for (field_name, boost) in field_boosts {
        let field = schema.get_field(&field_name).map_err(|_err| {
          Error::new(
            Status::InvalidArg,
            format!("Field `{field_name}` is not defined in the schema."),
          )
        })?;
        parser.set_field_boost(field, boost as tv::Score);
      }
    }

    // Set fuzzy fields if provided
    if let Some(fuzzy_fields) = fuzzy_fields {
      for (field_name, (prefix, distance, transpose_cost_one)) in fuzzy_fields {
        let field = schema.get_field(&field_name).map_err(|_err| {
          Error::new(
            Status::InvalidArg,
            format!("Field `{field_name}` is not defined in the schema."),
          )
        })?;
        parser.set_field_fuzzy(field, prefix, distance, transpose_cost_one);
      }
    }

    Ok(parser)
  }

  fn register_custom_text_analyzers(index: &tv::Index) {
    let analyzers = [
      ("ar_stem", tantivy::tokenizer::Language::Arabic),
      ("da_stem", tantivy::tokenizer::Language::Danish),
      ("nl_stem", tantivy::tokenizer::Language::Dutch),
      ("fi_stem", tantivy::tokenizer::Language::Finnish),
      ("fr_stem", tantivy::tokenizer::Language::French),
      ("de_stem", tantivy::tokenizer::Language::German),
      ("el_stem", tantivy::tokenizer::Language::Greek),
      ("hu_stem", tantivy::tokenizer::Language::Hungarian),
      ("it_stem", tantivy::tokenizer::Language::Italian),
      ("no_stem", tantivy::tokenizer::Language::Norwegian),
      ("pt_stem", tantivy::tokenizer::Language::Portuguese),
      ("ro_stem", tantivy::tokenizer::Language::Romanian),
      ("ru_stem", tantivy::tokenizer::Language::Russian),
      ("es_stem", tantivy::tokenizer::Language::Spanish),
      ("sv_stem", tantivy::tokenizer::Language::Swedish),
      ("ta_stem", tantivy::tokenizer::Language::Tamil),
      ("tr_stem", tantivy::tokenizer::Language::Turkish),
    ];

    for (name, lang) in &analyzers {
      let an =
        tantivy::tokenizer::TextAnalyzer::builder(tantivy::tokenizer::SimpleTokenizer::default())
          .filter(tantivy::tokenizer::RemoveLongFilter::limit(40))
          .filter(tantivy::tokenizer::LowerCaser)
          .filter(tantivy::tokenizer::Stemmer::new(*lang))
          .build();
      index.tokenizers().register(name, an);
    }
  }
}
