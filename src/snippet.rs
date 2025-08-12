use napi::bindgen_prelude::*;
use napi_derive::napi;
use tantivy as tv;
// Bring the trait into scope to use methods like `as_str()` on `OwnedValue`.
use tantivy::schema::Value;

/// Tantivy Snippet
///
/// Snippet contains a fragment of a document, and some highlighted
/// parts inside it.
#[napi]
pub struct Snippet {
    pub(crate) inner: tv::snippet::Snippet,
}

#[napi(object)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[napi]
impl Snippet {
    #[napi]
    pub fn to_html(&self) -> Result<String> {
        Ok(self.inner.to_html())
    }

    #[napi]
    pub fn highlighted(&self) -> Vec<Range> {
        let highlighted = self.inner.highlighted();
        let results = highlighted
            .iter()
            .map(|r| Range {
                start: r.start as u32,
                end: r.end as u32,
            })
            .collect::<Vec<_>>();
        results
    }

    #[napi]
    pub fn fragment(&self) -> Result<String> {
        Ok(self.inner.fragment().to_string())
    }
}

#[napi]
pub struct SnippetGenerator {
    pub(crate) field_name: String,
    pub(crate) inner: tv::snippet::SnippetGenerator,
}

#[napi]
impl SnippetGenerator {
    #[napi(factory)]
    pub fn create(
        searcher: &crate::Searcher,
        query: &crate::Query,
        schema: &crate::Schema,
        field_name: String,
    ) -> Result<SnippetGenerator> {
        let field = schema
            .inner
            .get_field(&field_name)
            .or(Err("field not found"))
            .map_err(|e| Error::from_reason(e))?;
        let generator = tv::snippet::SnippetGenerator::create(
            &searcher.inner,
            query.get(),
            field,
        )
        .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(SnippetGenerator {
            field_name,
            inner: generator,
        })
    }

    #[napi]
    pub fn snippet_from_doc(&self, doc: &crate::Document) -> crate::Snippet {
        let text: String = doc
            .iter_values_for_field(&self.field_name)
            .flat_map(|ov| ov.as_str())
            .collect::<Vec<&str>>()
            .join(" ");

        let result = self.inner.snippet(&text);
        Snippet { inner: result }
    }

    #[napi]
    pub fn set_max_num_chars(&mut self, max_num_chars: u32) {
        self.inner.set_max_num_chars(max_num_chars as usize);
    }
}
