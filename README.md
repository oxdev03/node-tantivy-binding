# node-tantivy-binding

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Node.js bindings for [Tantivy](https://github.com/quickwit-oss/tantivy), the full-text search engine library written in Rust.

This project is a Node.js port of [tantivy-py](https://github.com/quickwit-inc/tantivy-py), providing JavaScript/TypeScript bindings for the Tantivy search engine. The implementation closely follows the Python API to maintain consistency across language bindings.

⚠️ **Note**: This is a first draft implementation ported from tantivy-py (see submodule hash). There may be unidentified bugs. The test suite is also based on tantivy-py. Furthermore not all future api changes will be reflected in this binding.

# Installation

The bindings can be installed using npm:

```bash
npm install node-tantivy-binding
```

If no binary is present for your operating system, the bindings will be built from source, which requires Rust to be installed.

# Quick Start

```javascript
import { SchemaBuilder, FieldType, Index, Document } from 'node-tantivy-binding'

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
- **Flexible tokenization** and text analysis
- **JSON document support**

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
- Install `yarn`

## Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd node-tantivy-binding

# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```

## Testing

The project includes a comprehensive test suite migrated from tantivy-py:

```bash
npm test
```

## Project Status

This is a **first draft** port of tantivy-py to Node.js. While the core functionality works, please be aware:

- ⚠️ **Potential bugs**: Some edge cases may not be handled correctly
  - Due to different type casting
  - Missing error mapping
  - Usage of napi-rs v2 instead of v3
- 🔄 **API changes**: The API may evolve in future versions

## Architecture

Built with:

- **[napi-rs](https://napi.rs/)**: For Node.js ↔ Rust bindings
- **[Tantivy](https://github.com/quickwit-oss/tantivy)**: The underlying search engine
- **TypeScript**: Full type definitions included
- **Vitest**: For testing

## Acknowledgments

This project is heavily inspired by and based on:

- [tantivy-py](https://github.com/quickwit-inc/tantivy-py) - Python bindings for Tantivy
- [Tantivy](https://github.com/quickwit-oss/tantivy) - The core search engine library

## License

MIT License - see LICENSE file for details.
