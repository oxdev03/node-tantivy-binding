use napi_derive::napi;
use napi::{Result, Error, Status};
use tantivy::schema::Facet as TantivyFacet;

/// Represents a facet in Tantivy.
/// 
/// Facets are used for hierarchical faceting, allowing you to organize documents
/// into a tree-like structure for filtering and navigation.
/// 
/// Example:
/// ```javascript
/// const facet = Facet.fromText("/category/electronics/smartphones");
/// console.log(facet.toString()); // "/category/electronics/smartphones"
/// ```
#[napi]
pub struct Facet {
    pub(crate) inner: TantivyFacet,
}

#[napi]
impl Facet {
    /// Create a facet from a text representation.
    /// 
    /// @param facet_string - The facet path as a string (e.g., "/category/electronics/phones")
    /// @returns A new Facet instance
    #[napi(factory)]
    pub fn from_text(facet_string: String) -> Result<Facet> {
        let facet = TantivyFacet::from_text(&facet_string).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Invalid facet string: {}", e),
            )
        })?;
        Ok(Facet { inner: facet })
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

    /// Convert the facet to its string representation.
    /// 
    /// @returns The facet path as a string
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Get the path segments of the facet.
    /// 
    /// @returns Array of path segments
    #[napi]
    pub fn to_path(&self) -> Vec<String> {
        self.inner.to_path().map(|s| s.to_string()).collect()
    }

    /// Check if this facet is a prefix of another facet.
    /// 
    /// @param other - The other facet to compare against
    /// @returns True if this facet is a prefix of the other
    #[napi]
    pub fn is_prefix_of(&self, other: &Facet) -> bool {
        self.inner.is_prefix_of(&other.inner)
    }

    /// Get the parent facet (one level up in the hierarchy).
    /// 
    /// @returns The parent facet, or null if this is the root
    #[napi]
    pub fn parent(&self) -> Option<Facet> {
        self.inner.parent().map(|parent| Facet { inner: parent })
    }

    /// Get the depth of the facet (number of path segments).
    /// 
    /// @returns The depth of the facet
    #[napi]
    pub fn depth(&self) -> u32 {
        self.inner.to_path().len() as u32
    }
}

impl Facet {
    pub(crate) fn new(facet: TantivyFacet) -> Self {
        Self { inner: facet }
    }
}
