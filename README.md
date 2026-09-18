# node-tantivy-binding

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Node.js bindings for [Tantivy](https://github.com/quickwit-oss/tantivy), the full-text search engine library written in Rust.

This project is a Node.js port of [tantivy-py](https://github.com/quickwit-inc/tantivy-py), providing JavaScript/TypeScript bindings for the Tantivy search engine. The implementation closely follows the Python API to maintain consistency across language bindings.

# Installation

The bindings can be installed using npm:

```bash
npm install @oxdev03-org/node-tantivy-binding
```

If no binary is present for your operating system, the bindings will be built from source, which requires Rust to be installed.

# Quick Start

For more detailed examples, see the [tutorials](./docs/tutorials.md).

```javascript
import { SchemaBuilder, FieldType, Index, Document } from '@oxdev03-org/node-tantivy-binding'

// Create a schema
const schema = new SchemaBuilder()
  .addTextField('title', { stored: true })
  .addTextField('body', { stored: true })
  .build()

// Create an index
const index = new Index(schema)
const writer = index.writer()

// Add documents
const doc1 = new Document()
doc1.addText('title', 'The Old Man and the Sea')
doc1.addText('body', 'He was an old man who fished alone in a skiff in the Gulf Stream.')
writer.addDocument(doc1)
writer.commit()

// Search
const searcher = index.searcher()
const query = index.parseQuery('sea', ['title', 'body'])
const results = searcher.search(query, 10)

console.log('Found', results.hits.length, 'results')
```

# Features

This Node.js binding provides access to most of Tantivy's functionality:

- **Full-text search** with BM25 scoring
- **Structured queries** with boolean operations
- **Faceted search** for filtering and aggregation
- **Snippet generation** for search result highlighting
- **Query explanation** for debugging relevance scoring
- **Multiple field types**: text, integers, floats, dates, facets
- **Flexible tokenization** and text analysis, including fast-field tokenizers
- **JSON document support**, with optional dot-path expansion
- **Aggregations**, including cardinality
- **Fast-field reads** (`fastFieldValues`) and prefix term lookup (`termsWithPrefix`)
- **Schema-free query parsing** via `parseQuery` / `parseQueryLenient`

## API Compatibility

The API closely follows [tantivy-py](https://github.com/quickwit-inc/tantivy-py) to maintain consistency:

- Same class names and method signatures where possible
- Compatible document and query structures
- Equivalent search result formats
- Similar configuration options

# Development

## Requirements

- Install the latest `Rust` (required for building from source)
- Install `Node.js@22+` which fully supports `Node-API`
- Install `pnpm`

## Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd node-tantivy-binding

# Install dependencies
pnpm install

# Build the native module
pnpm build

# Run tests
pnpm test
```

## Testing

The project includes a comprehensive test suite migrated from tantivy-py:

```bash
pnpm test
```

## Project Status

This library provides stable Node.js bindings for Tantivy, originally ported from [tantivy-py](https://github.com/quickwit-inc/tantivy-py). The core API is fully functional and covered by a comprehensive test suite.

### Known Implementation Differences

The Node.js implementation differs from the Python version in several ways:

#### 🔴 Critical Validation Issues

##### Multi-valued Single Fields (Too Lenient)

**Current behavior**: an array is accepted wherever a single value is expected
**TODO**: decide whether to match Python, which rejects it

```javascript
// ❌ This currently PASSES in Node.js but should FAIL:
Document.fromDict({ unsigned: [1000, 50] }, schema) // Should reject arrays for single fields
```

Numeric values themselves are validated as in tantivy-py: a negative value for an
unsigned field, or a fractional value for an integer field, throws rather than
being silently coerced. Values beyond `Number.MAX_SAFE_INTEGER` can be passed as
a `BigInt`.

##### Bytes Field Validation (Too Restrictive)

**Current behavior**: Only accepts Buffer objects
**TODO**: Support byte arrays like Python version

```javascript
// ❌ These currently FAIL in Node.js but should PASS:
Document.fromDict({ bytes: [1, 2, 3] }, schema) // Should accept byte arrays
Document.fromDict(
  {
    bytes: [
      [1, 2, 3],
      [4, 5, 6],
    ],
  },
  schema,
) // Should accept nested arrays
```

##### JSON Field Validation (Too Lenient)

**Current behavior**: Accepts primitive types for JSON fields  
**TODO**: Restrict to objects/arrays only

```javascript
// ❌ These currently PASS in Node.js but should FAIL:
Document.fromDict({ json: 123 }, schema) // Should reject numbers
Document.fromDict({ json: 'hello' }, schema) // Should reject strings
```

#### 🔵 Type System Differences

##### Date Handling

**Current**: Dates cross the boundary as millisecond timestamps. `Document.fromDict()`
and the query builders also accept a JavaScript `Date` or an ISO 8601 string, and
`toDict()` returns milliseconds since the epoch.
**Python**: Uses `datetime` objects with nanosecond storage.

Sub-millisecond precision is therefore not representable from JavaScript.

#### 🔵 Resource Management

`IndexWriter` has no equivalent of tantivy-py's `with index.writer() as writer:`
block, because JavaScript has no deterministic destructor. Call
`writer.waitMergingThreads()` when you are done with a writer — it commits nothing
on its own, so commit first, and it is what releases the directory lock:

```javascript
const writer = index.writer()
try {
  writer.addDocument(doc)
  writer.commit()
} finally {
  writer.waitMergingThreads()
}
```

#### 🔵 Concurrency

tantivy-py releases the GIL around the blocking calls (`addDocument`, `commit`,
`search`, …). Node has no GIL, but these methods run synchronously on the main
thread and are not offloaded to the libuv thread pool.

#### 🔵 Naming

Two shapes differ from tantivy-py because napi-rs models them differently:

- Static constructors live on a companion class: `FilterStatic.lowercase()` and
  `TokenizerStatic.simple()` rather than `Filter.lowercase()` / `Tokenizer.simple()`.
- `DocAddress`, `SearchResult` and `SearchHit` are plain objects rather than
  classes, so they have no constructors or getters — read `hit.docAddress`,
  `hit.score` and `hit.order` directly.

`FieldType` uses tantivy-py's variant names (`Text`, `Unsigned`, `Integer`,
`Float`, `Boolean`, `Json`), not tantivy's Rust `Type` names.

Python's pickle hooks (`__reduce__`, `__getnewargs__`) have no counterpart;
`Schema` exposes `toJSON()` / `Schema.fromJson()` instead.

## Architecture

Built with:

- **[napi-rs](https://napi.rs/)**: For Node.js bindings
- **[Tantivy](https://github.com/quickwit-oss/tantivy)**: The underlying search engine

## Acknowledgments

This project is heavily inspired by and based on:

- [tantivy-py](https://github.com/quickwit-inc/tantivy-py) - Python bindings for Tantivy
- [Tantivy](https://github.com/quickwit-oss/tantivy) - The core search engine library

## License

MIT License - see LICENSE file for details.
