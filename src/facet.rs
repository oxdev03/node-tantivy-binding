use napi::{Error, Result, Status};
use napi_derive::napi;
use tantivy::schema::Facet as TantivyFacet;

/// Represents a facet in Tantivy.
///
/// A Facet represent a point in a given hierarchy.
/// They are typically represented similarly to a filepath. For instance, an
/// e-commerce website could have a Facet for /electronics/tv_and_video/led_tv.
///
/// A document can be associated to any number of facets. The hierarchy
/// implicitely imply that a document belonging to a facet also belongs to the
/// ancestor of its facet. In the example above, /electronics/tv_and_video/
/// and /electronics.
///
/// Example:
/// ```javascript
/// const facet = Facet.fromString("/category/electronics/smartphones");
/// console.log(facet.toPathStr()); // "/category/electronics/smartphones"
/// ```
#[napi]
pub struct Facet {
  pub(crate) inner: TantivyFacet,
}

#[napi]
impl Facet {
  /// Creates a `Facet` from its binary representation.
  #[napi(factory)]
  pub fn from_encoded(encoded_bytes: Vec<u8>) -> Result<Facet> {
    let inner = TantivyFacet::from_encoded(encoded_bytes)
      .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid encoded facet: {}", e)))?;
    Ok(Facet { inner })
  }

  /// Create a new instance of the "root facet" Equivalent to /.
  #[napi(factory)]
  pub fn root() -> Facet {
    Facet {
      inner: TantivyFacet::root(),
    }
  }

  /// Returns true if the facet is the root facet /.
  #[napi(getter)]
  pub fn is_root(&self) -> bool {
    self.inner.is_root()
  }

  /// Returns true if another Facet is a subfacet of this facet.
  ///
  /// @param other - The Facet that we should check if this facet is a subset of.
  /// @returns True if this facet is a prefix of the other
  #[napi]
  pub fn is_prefix_of(&self, other: &Facet) -> bool {
    self.inner.is_prefix_of(&other.inner)
  }

  /// Create a Facet object from a string.
  ///
  /// @param facet_string - The string that contains a facet.
  /// @returns The created Facet.
  #[napi(factory)]
  pub fn from_string(facet_string: String) -> Facet {
    Facet {
      inner: TantivyFacet::from(facet_string.as_str()),
    }
  }

  /// Create a facet from an array of path segments.
  ///
  /// @param path - Array of path segments (e.g., ["category", "electronics", "phones"])
  /// @returns A new Facet instance
  #[napi(factory)]
  pub fn from_path(path: Vec<String>) -> Facet {
    let facet = TantivyFacet::from_path(path.iter().map(|s| s.as_str()));
    Facet { inner: facet }
  }

  /// Returns the list of `segments` that forms a facet path.
  ///
  /// For instance `//europe/france` becomes `["europe", "france"]`.
  /// @returns Array of path segments
  #[napi]
  pub fn to_path(&self) -> Vec<String> {
    self
      .inner
      .to_path()
      .into_iter()
      .map(|s| s.to_string())
      .collect()
  }

  /// Returns the facet string representation.
  ///
  /// @returns The facet path as a string
  #[napi]
  pub fn to_path_str(&self) -> String {
    self.inner.to_string()
  }

  /// Convert the facet to its string representation.
  ///
  /// @returns The facet path as a string
  #[napi]
  pub fn to_string(&self) -> String {
    self.inner.to_string()
  }
}

impl Facet {
  pub(crate) fn new(facet: TantivyFacet) -> Self {
    Self { inner: facet }
  }
}
