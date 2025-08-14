use crate::schema::Schema;
use napi::{Error, Result, Status};
use napi_derive::napi;
use tantivy::schema::{
  BytesOptions, DateOptions, IndexRecordOption, IpAddrOptions, NumericOptions,
  Schema as TantivySchema, SchemaBuilder as TantivySchemaBuilder, TextFieldIndexing, TextOptions,
  INDEXED,
};

/// A SchemaBuilder can be used to create a Schema.
///
/// The schema will list all the fields and associated types and options.
///
/// Example:
/// ```javascript
/// const builder = new SchemaBuilder();
/// builder.addTextField("title", { stored: true });
/// builder.addTextField("body", { stored: false });
/// builder.addIntegerField("year", { stored: true, indexed: true });
/// const schema = builder.build();
/// ```
#[napi]
pub struct SchemaBuilder {
  inner: Option<TantivySchemaBuilder>,
}

/// Text field indexing options
#[napi(object)]
pub struct TextFieldOptions {
  /// Store the field value (can be retrieved from search results)
  pub stored: Option<bool>,
  /// Fast field access (column-oriented storage)
  pub fast: Option<bool>,
  /// Tokenizer name to use (default: "default")
  pub tokenizer_name: Option<String>,
  /// Index record option: "basic", "freq", or "position" (default: "position")
  pub index_option: Option<String>,
}

/// Numeric field options (for integers, floats, dates)
#[napi(object)]
pub struct NumericFieldOptions {
  /// Store the field value (can be retrieved from search results)
  pub stored: Option<bool>,
  /// Index the field (enables searching)
  pub indexed: Option<bool>,
  /// Fast field access (column-oriented storage)
  pub fast: Option<bool>,
}

/// Bytes field options
#[napi(object)]
pub struct BytesFieldOptions {
  /// Store the field value (can be retrieved from search results)
  pub stored: Option<bool>,
  /// Index the field (enables searching)
  pub indexed: Option<bool>,
  /// Fast field access (column-oriented storage)
  pub fast: Option<bool>,
}

/// IP address field options
#[napi(object)]
pub struct IpAddrFieldOptions {
  /// Store the field value (can be retrieved from search results)
  pub stored: Option<bool>,
  /// Index the field (enables searching)
  pub indexed: Option<bool>,
  /// Fast field access (column-oriented storage)
  pub fast: Option<bool>,
}

#[napi]
impl SchemaBuilder {
  /// Create a new SchemaBuilder.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(TantivySchema::builder()),
    }
  }

  /// Check if a field name is valid.
  ///
  /// @param name - The field name to validate
  /// @returns True if the field name is valid
  #[napi]
  pub fn is_valid_field_name(name: String) -> bool {
    tantivy::schema::is_valid_field_name(&name)
  }

  /// Add a text field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Text field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_text_field(
    &mut self,
    name: String,
    options: Option<TextFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_text_options(options)?;
    builder.add_text_field(&name, opts);
    Ok(self)
  }

  /// Add a signed integer field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Numeric field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_integer_field(
    &mut self,
    name: String,
    options: Option<NumericFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_numeric_options(options);
    builder.add_i64_field(&name, opts);
    Ok(self)
  }

  /// Add an unsigned integer field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Numeric field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_unsigned_field(
    &mut self,
    name: String,
    options: Option<NumericFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_numeric_options(options);
    builder.add_u64_field(&name, opts);
    Ok(self)
  }

  /// Add a float field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Numeric field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_float_field(
    &mut self,
    name: String,
    options: Option<NumericFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_numeric_options(options);
    builder.add_f64_field(&name, opts);
    Ok(self)
  }

  /// Add a boolean field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Numeric field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_boolean_field(
    &mut self,
    name: String,
    options: Option<NumericFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_numeric_options(options);
    builder.add_bool_field(&name, opts);
    Ok(self)
  }

  /// Add a date field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Numeric field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_date_field(
    &mut self,
    name: String,
    options: Option<NumericFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_date_options(options);
    builder.add_date_field(&name, opts);
    Ok(self)
  }

  /// Add a JSON field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - JSON field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_json_field(
    &mut self,
    name: String,
    options: Option<TextFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_text_options(options)?;
    builder.add_json_field(&name, opts);
    Ok(self)
  }

  /// Add a facet field to the schema.
  ///
  /// @param name - The name of the field
  /// @returns Self for method chaining
  #[napi]
  pub fn add_facet_field(&mut self, name: String) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    builder.add_facet_field(&name, INDEXED);
    Ok(self)
  }

  /// Add a bytes field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - Bytes field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_bytes_field(
    &mut self,
    name: String,
    options: Option<BytesFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_bytes_options(options);
    builder.add_bytes_field(&name, opts);
    Ok(self)
  }

  /// Add an IP address field to the schema.
  ///
  /// @param name - The name of the field
  /// @param options - IP address field options
  /// @returns Self for method chaining
  #[napi]
  pub fn add_ip_addr_field(
    &mut self,
    name: String,
    options: Option<IpAddrFieldOptions>,
  ) -> Result<&Self> {
    let builder = self
      .inner
      .as_mut()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let opts = Self::build_ip_addr_options(options);
    builder.add_ip_addr_field(&name, opts);
    Ok(self)
  }

  /// Build the final schema.
  ///
  /// After calling this method, the SchemaBuilder can no longer be used.
  ///
  /// @returns The built schema
  #[napi]
  pub fn build(&mut self) -> Result<Schema> {
    let builder = self
      .inner
      .take()
      .ok_or_else(|| Error::new(Status::InvalidArg, "Schema builder is no longer valid"))?;

    let schema = builder.build();
    Ok(Schema::new(schema))
  }
}

impl SchemaBuilder {
  fn build_numeric_options(options: Option<NumericFieldOptions>) -> NumericOptions {
    let mut opts = NumericOptions::default();

    if let Some(options) = options {
      if options.stored.unwrap_or(false) {
        opts = opts.set_stored();
      }
      if options.indexed.unwrap_or(false) {
        opts = opts.set_indexed();
      }
      if options.fast.unwrap_or(false) {
        opts = opts.set_fast();
      }
    }

    opts
  }

  fn build_date_options(options: Option<NumericFieldOptions>) -> DateOptions {
    let mut opts = DateOptions::default();

    if let Some(options) = options {
      if options.stored.unwrap_or(false) {
        opts = opts.set_stored();
      }
      if options.indexed.unwrap_or(false) {
        opts = opts.set_indexed();
      }
      if options.fast.unwrap_or(false) {
        opts = opts.set_fast();
      }
    }

    opts
  }

  fn build_bytes_options(options: Option<BytesFieldOptions>) -> BytesOptions {
    let mut opts = BytesOptions::default();

    if let Some(options) = options {
      if options.stored.unwrap_or(false) {
        opts = opts.set_stored();
      }
      if options.indexed.unwrap_or(false) {
        opts = opts.set_indexed();
      }
      if options.fast.unwrap_or(false) {
        opts = opts.set_fast();
      }
    }

    opts
  }

  fn build_ip_addr_options(options: Option<IpAddrFieldOptions>) -> IpAddrOptions {
    let mut opts = IpAddrOptions::default();

    if let Some(options) = options {
      if options.stored.unwrap_or(false) {
        opts = opts.set_stored();
      }
      if options.indexed.unwrap_or(false) {
        opts = opts.set_indexed();
      }
      if options.fast.unwrap_or(false) {
        opts = opts.set_fast();
      }
    }

    opts
  }

  fn build_text_options(options: Option<TextFieldOptions>) -> Result<TextOptions> {
    let stored = options.as_ref().and_then(|o| o.stored).unwrap_or(false);
    let fast = options.as_ref().and_then(|o| o.fast).unwrap_or(false);
    let tokenizer_name = options
      .as_ref()
      .and_then(|o| o.tokenizer_name.as_deref())
      .unwrap_or("default");
    let index_option = options
      .as_ref()
      .and_then(|o| o.index_option.as_deref())
      .unwrap_or("position");

    let index_record_option = match index_option {
      "position" => IndexRecordOption::WithFreqsAndPositions,
      "freq" => IndexRecordOption::WithFreqs,
      "basic" => IndexRecordOption::Basic,
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          "Invalid index option, valid choices are: 'basic', 'freq', and 'position'",
        ))
      }
    };

    let indexing = TextFieldIndexing::default()
      .set_tokenizer(tokenizer_name)
      .set_index_option(index_record_option);

    let mut text_options = TextOptions::default().set_indexing_options(indexing);

    if stored {
      text_options = text_options.set_stored();
    }

    if fast {
      let text_tokenizer = if tokenizer_name != "raw" {
        Some(tokenizer_name)
      } else {
        None
      };
      text_options = text_options.set_fast(text_tokenizer);
    }

    Ok(text_options)
  }
}
