use napi::bindgen_prelude::*;
use napi_derive::napi;
use tantivy::tokenizer as tvt;

/// All Tantivy's built-in tokenizers in one place.
/// Each static method, e.g. Tokenizer.simple(),
/// creates a wrapper around a Tantivy tokenizer.
///
/// ## Example:
///
/// ```javascript
/// tokenizer = Tokenizer.simple()
/// ```
///
/// ## Usage
///
/// In general, tokenizer objects' only reason
/// for existing is to be passed to
/// TextAnalyzerBuilder(tokenizer=<tokenizer>)
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
///
#[derive(Debug, Clone)]
pub(crate) enum TokenizerType {
  Raw,
  Simple,
  Whitespace,
  Regex {
    pattern: String,
  },
  Ngram {
    min_gram: u32,
    max_gram: u32,
    prefix_only: bool,
  },
  Facet,
}

#[napi]
#[derive(Debug, Clone)]
pub struct Tokenizer {
  pub(crate) inner: TokenizerType,
}

#[napi]
pub struct TokenizerStatic;

#[napi]
impl TokenizerStatic {
  /// SimpleTokenizer
  #[napi]
  pub fn simple() -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Simple,
    }
  }

  /// WhitespaceTokenizer
  #[napi]
  pub fn whitespace() -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Whitespace,
    }
  }

  /// Raw Tokenizer
  #[napi]
  pub fn raw() -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Raw,
    }
  }

  /// FacetTokenizer
  #[napi]
  pub fn facet() -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Facet,
    }
  }

  /// Regextokenizer
  #[napi]
  pub fn regex(pattern: String) -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Regex { pattern },
    }
  }

  /// NgramTokenizer
  ///
  /// @param minGram - Minimum character length of each ngram.
  /// @param maxGram - Maximum character length of each ngram.
  /// @param prefixOnly - If true, ngrams must count from the start of the word.
  #[napi]
  pub fn ngram(
    min_gram: Option<u32>,
    max_gram: Option<u32>,
    prefix_only: Option<bool>,
  ) -> Tokenizer {
    Tokenizer {
      inner: TokenizerType::Ngram {
        min_gram: min_gram.unwrap_or(2),
        max_gram: max_gram.unwrap_or(3),
        prefix_only: prefix_only.unwrap_or(false),
      },
    }
  }
}

/// All Tantivy's builtin TokenFilters.
///
/// ## Example
///
/// ```javascript
/// filter = Filter.alphanumOnly()
/// ```
///
/// ## Usage
///
/// In general, filter objects exist to
/// be passed to the filter() method
/// of a TextAnalyzerBuilder instance.
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
///
#[derive(Debug, Clone)]
pub(crate) enum FilterType {
  AlphaNumOnly,
  AsciiFolding,
  LowerCaser,
  RemoveLong { length_limit: u32 },
  Stemmer { language: String },
  StopWord { language: String },
  CustomStopWord { stopwords: Vec<String> },
  SplitCompound { constituent_words: Vec<String> },
}

#[napi]
#[derive(Debug, Clone)]
pub struct Filter {
  pub(crate) inner: FilterType,
}

#[napi]
pub struct FilterStatic;

#[napi]
impl FilterStatic {
  /// AlphaNumOnlyFilter
  #[napi]
  pub fn alphanum_only() -> Filter {
    Filter {
      inner: FilterType::AlphaNumOnly,
    }
  }

  /// AsciiFoldingFilter
  #[napi]
  pub fn ascii_fold() -> Filter {
    Filter {
      inner: FilterType::AsciiFolding,
    }
  }

  #[napi]
  pub fn lowercase() -> Filter {
    Filter {
      inner: FilterType::LowerCaser,
    }
  }

  /// RemoveLongFilter
  ///
  /// @param lengthLimit - max character length of token.
  #[napi]
  pub fn remove_long(length_limit: u32) -> Filter {
    Filter {
      inner: FilterType::RemoveLong { length_limit },
    }
  }

  /// Stemmer
  #[napi]
  pub fn stemmer(language: String) -> Filter {
    Filter {
      inner: FilterType::Stemmer { language },
    }
  }

  /// StopWordFilter (builtin stop word list)
  ///
  /// @param language - Stop words list language.
  ///   Valid values: {
  ///     "arabic", "danish", "dutch", "english", "finnish", "french", "german", "greek",
  ///     "hungarian", "italian", "norwegian", "portuguese", "romanian", "russian",
  ///     "spanish", "swedish", "tamil", "turkish"
  ///   }
  #[napi]
  pub fn stopword(language: String) -> Filter {
    Filter {
      inner: FilterType::StopWord { language },
    }
  }

  /// StopWordFilter (user-provided stop word list)
  ///
  /// This variant of Filter.stopword() lets you provide
  /// your own custom list of stopwords.
  ///
  /// @param stopwords - a list of words to be removed.
  #[napi]
  pub fn custom_stopword(stopwords: Vec<String>) -> Filter {
    Filter {
      inner: FilterType::CustomStopWord { stopwords },
    }
  }

  /// SplitCompoundWords
  ///
  /// https://docs.rs/tantivy/latest/tantivy/tokenizer/struct.SplitCompoundWords.html
  ///
  /// @param constituentWords - words that make up compound word (must be in order).
  ///
  /// Example:
  ///
  /// ```javascript
  /// // useless, contrived example:
  /// compound_spliter = Filter.splitCompound(['butter', 'fly'])
  /// // Will split 'butterfly' -> ['butter', 'fly'],
  /// // but won't split 'buttering' or 'buttercupfly'
  /// ```
  #[napi]
  pub fn split_compound(constituent_words: Vec<String>) -> Filter {
    Filter {
      inner: FilterType::SplitCompound { constituent_words },
    }
  }
}

fn parse_language(lang: &str) -> Result<tvt::Language> {
  match lang.to_lowercase().as_str() {
    "arabic" => Ok(tvt::Language::Arabic),
    "danish" => Ok(tvt::Language::Danish),
    "dutch" => Ok(tvt::Language::Dutch),
    "english" => Ok(tvt::Language::English),
    "finnish" => Ok(tvt::Language::Finnish),
    "french" => Ok(tvt::Language::French),
    "german" => Ok(tvt::Language::German),
    "greek" => Ok(tvt::Language::Greek),
    "hungarian" => Ok(tvt::Language::Hungarian),
    "italian" => Ok(tvt::Language::Italian),
    "norwegian" => Ok(tvt::Language::Norwegian),
    "portuguese" => Ok(tvt::Language::Portuguese),
    "romanian" => Ok(tvt::Language::Romanian),
    "russian" => Ok(tvt::Language::Russian),
    "spanish" => Ok(tvt::Language::Spanish),
    "swedish" => Ok(tvt::Language::Swedish),
    "tamil" => Ok(tvt::Language::Tamil),
    "turkish" => Ok(tvt::Language::Turkish),
    _ => Err(Error::from_reason(format!(
      "Unsupported language: {}",
      lang
    ))),
  }
}

/// Tantivy's TextAnalyzer
///
/// Do not instantiate this class directly.
/// Use the `TextAnalyzerBuilder` class instead.
#[napi]
#[derive(Clone)]
pub struct TextAnalyzer {
  pub(crate) analyzer: tvt::TextAnalyzer,
}

#[napi]
impl TextAnalyzer {
  /// Tokenize a string
  /// @param text - text to tokenize.
  /// @returns - a list of tokens/words.
  #[napi]
  pub fn analyze(&mut self, text: String) -> Vec<String> {
    let mut token_stream = self.analyzer.token_stream(&text);
    let mut tokens = Vec::new();

    while token_stream.advance() {
      tokens.push(token_stream.token().text.clone());
    }
    tokens
  }
}

/// Tantivy's TextAnalyzerBuilder
///
/// # Example
///
/// ```javascript
/// my_analyzer = new TextAnalyzerBuilder(Tokenizer.simple())
///     .filter(Filter.lowercase())
///     .filter(Filter.ngram())
///     .build()
/// ```
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/struct.TextAnalyzerBuilder.html
#[napi]
pub struct TextAnalyzerBuilder {
  builder: Option<tvt::TextAnalyzerBuilder>,
}

#[napi]
impl TextAnalyzerBuilder {
  #[napi(constructor)]
  pub fn new(tokenizer: &Tokenizer) -> Result<Self> {
    let builder: tvt::TextAnalyzerBuilder = match &tokenizer.inner {
      TokenizerType::Raw => tvt::TextAnalyzer::builder(tvt::RawTokenizer::default()).dynamic(),
      TokenizerType::Simple => {
        tvt::TextAnalyzer::builder(tvt::SimpleTokenizer::default()).dynamic()
      }
      TokenizerType::Whitespace => {
        tvt::TextAnalyzer::builder(tvt::WhitespaceTokenizer::default()).dynamic()
      }
      TokenizerType::Regex { pattern } => tvt::TextAnalyzer::builder(
        tvt::RegexTokenizer::new(pattern)
          .map_err(|e| Error::from_reason(format!("Invalid regex pattern: {}", e)))?,
      )
      .dynamic(),
      TokenizerType::Ngram {
        min_gram,
        max_gram,
        prefix_only,
      } => tvt::TextAnalyzer::builder(
        tvt::NgramTokenizer::new(*min_gram as usize, *max_gram as usize, *prefix_only).unwrap(),
      )
      .dynamic(),
      TokenizerType::Facet => tvt::TextAnalyzer::builder(tvt::FacetTokenizer::default()).dynamic(),
    };

    Ok(TextAnalyzerBuilder {
      builder: Some(builder.dynamic()),
    })
  }

  /// Add filter to the builder.
  ///
  /// @param filter - a Filter object.
  /// @returns - A new instance of the builder
  ///
  /// Note: The builder is _not_ mutated in-place.
  #[napi]
  pub fn filter(&mut self, filter: &Filter) -> Result<Self> {
    if let Some(builder) = self.builder.take() {
      let new_builder: tvt::TextAnalyzerBuilder = match &filter.inner {
        FilterType::AlphaNumOnly => builder.filter_dynamic(tvt::AlphaNumOnlyFilter {}),
        FilterType::AsciiFolding => builder.filter_dynamic(tvt::AsciiFoldingFilter),
        FilterType::LowerCaser => builder.filter_dynamic(tvt::LowerCaser),
        FilterType::RemoveLong { length_limit } => {
          builder.filter_dynamic(tvt::RemoveLongFilter::limit(*length_limit as usize))
        }
        FilterType::Stemmer { language } => match parse_language(language) {
          Ok(lang) => builder.filter_dynamic(tvt::Stemmer::new(lang)),
          Err(e) => return Err(e),
        },
        FilterType::StopWord { language } => match parse_language(language) {
          Ok(lang) => builder.filter_dynamic(tvt::StopWordFilter::new(lang).unwrap()),
          Err(e) => return Err(e),
        },
        FilterType::CustomStopWord { stopwords } => {
          builder.filter_dynamic(tvt::StopWordFilter::remove(stopwords.clone()))
        }
        FilterType::SplitCompound { constituent_words } => builder
          .filter_dynamic(tvt::SplitCompoundWords::from_dictionary(constituent_words).unwrap()),
      };
      Ok(TextAnalyzerBuilder {
        builder: Some(new_builder),
      })
    } else {
      Err(Error::from_reason("Builder has already been consumed"))
    }
  }

  /// Build final TextAnalyzer object.
  ///
  /// @returns - TextAnalyzer with tokenizer and filters baked in.
  ///
  /// Tip: TextAnalyzer's `analyze(text) -> tokens` method lets you
  /// easily check if your analyzer is working as expected.
  #[napi]
  pub fn build(&mut self) -> Result<TextAnalyzer> {
    if let Some(builder) = self.builder.take() {
      Ok(TextAnalyzer {
        analyzer: builder.build(),
      })
    } else {
      Err(Error::from_reason("Builder has already been consumed"))
    }
  }
}
