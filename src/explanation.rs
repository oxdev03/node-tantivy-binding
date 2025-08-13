use napi_derive::napi;
use tantivy as tv;

/// Represents an explanation of how a document matched a query.
/// This provides detailed scoring information and query analysis.
#[napi]
pub struct Explanation {
  pub(crate) inner: tv::query::Explanation,
}

impl Explanation {
  pub(crate) fn new(inner: tv::query::Explanation) -> Self {
    Explanation { inner }
  }
}

#[napi]
impl Explanation {
  /// Returns a JSON representation of the explanation.
  /// This contains detailed information about how the document matched the query
  /// and how the score was calculated.
  #[napi]
  pub fn to_json(&self) -> String {
    self.inner.to_pretty_json()
  }

  /// Returns a string representation of the explanation.
  #[napi]
  pub fn to_string(&self) -> String {
    format!("Explanation(value={})", self.inner.value())
  }

  /// Gets the score value from the explanation.
  #[napi]
  pub fn value(&self) -> f32 {
    self.inner.value()
  }
}
