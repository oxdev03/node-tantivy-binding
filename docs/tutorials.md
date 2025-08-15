# Tutorials

> Credits: The doc content is based on [tantivy-py](https://github.com/quickwit-oss/tantivy-py/blob/master/docs/tutorials.md) documentation.

## Building an index and populating it

<!-- example:basic-indexing source:../examples/basic-indexing.ts -->

```typescript
import { SchemaBuilder, Index } from '../index'

// Declaring our schema.
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

// Creating our index (in memory)
const index = new Index(schema)

// Assertions
console.assert(index !== undefined, 'Index should be created')
console.assert(schema !== undefined, 'Schema should be built')
```

To have a persistent index, use the path parameter to store the index on the disk, e.g:

<!-- example:persistent-index source:../examples/persistent-index.ts -->

```typescript
import { SchemaBuilder, Index } from '../index'
import * as os from 'os'
import * as path from 'path'
import * as fs from 'fs'

const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tantivy-'))
const indexPath = path.join(tmpDir, 'index')
fs.mkdirSync(indexPath, { recursive: true })
const persistentIndex = new Index(schema, indexPath)

// Assertions
console.assert(fs.existsSync(indexPath), 'Index directory should exist')

const writer = persistentIndex.writer()
console.assert(writer !== undefined, 'Writer should be created for persistent index')

// Cleanup
try {
  fs.rmSync(tmpDir, { recursive: true, force: true })
} catch {}
```

By default, tantivy offers the following tokenizers which can be used in node-tantivy:

- `default` - The tokenizer that will be used if you do not assign a specific tokenizer to your text field. It will chop your text on punctuation and whitespaces, removes tokens that are longer than 40 chars, and lowercase your text.

- `raw` - Does not actually tokenize your text. It keeps it entirely unprocessed. It can be useful to index uuids, or urls for instance.

- `en_stem` - In addition to what `default` does, the `en_stem` tokenizer also applies stemming to your tokens. Stemming consists in trimming words to remove their inflection. This tokenizer is slower than the default one, but is recommended to improve recall.

To use the above tokenizers, simply provide them as a parameter to `addTextField`:

<!-- example:custom-tokenizer source:../examples/custom-tokenizer.ts -->

```typescript
import { SchemaBuilder } from '../index'

const schemaBuilderTok = new SchemaBuilder()
schemaBuilderTok.addTextField('body', { stored: true, tokenizerName: 'en_stem' })
const schema = schemaBuilderTok.build()

// Assertions
// Test that the field was added with the correct tokenizer
const schemaJson = JSON.stringify(schema)
console.assert(schemaJson.includes('body'), 'Schema should contain body field')
```

## Adding one document

<!-- example:adding-document source:../examples/adding-document.ts -->

```typescript
import { SchemaBuilder, Index, Document } from '../index'

const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
// ... and committing
writer.commit()
writer.waitMergingThreads()

// Assertions
console.assert(index !== undefined, 'Index should be created')
console.assert(doc !== undefined, 'Document should be created')
console.assert(schema !== undefined, 'Schema should be built')

// Verify document was added by searching
index.reload()
const searcher = index.searcher()
const query = index.parseQuery('sea', ['title'])
const results = searcher.search(query, 10)

console.assert(results.hits.length === 1, 'Should find one document')
const foundDoc = searcher.doc(results.hits[0].docAddress)
const foundDocDict = foundDoc.toDict() as any

console.assert(foundDocDict.title[0] === 'The Old Man and the Sea', 'Document title should match')
console.assert(foundDocDict.doc_id[0] === 1, 'Document ID should be 1')
```

Note that `waitMergingThreads()` must come at the end, because the `writer` object will not be usable after this call.

## Building and Executing Queries with the Query Parser

With the Query Parser, you can easily build simple queries for your index.

First you need to get a searcher for the index:

<!-- example:basic-search source:../examples/basic-search.ts -->

```typescript
import { SchemaBuilder, Index, Document } from '../index'

// Setup index and add document (from previous examples)
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
writer.commit()
writer.waitMergingThreads()

// Reload the index to ensure it points to the last commit
index.reload()
const searcher = index.searcher()

// Parse and execute query
const query = index.parseQuery('fish days', ['title', 'body'])
const searchResults = searcher.search(query, 3)
const bestHit = searchResults.hits[0]
const bestDoc = searcher.doc(bestHit.docAddress)
const bestDocDict = bestDoc.toDict() as any

console.log('Best document title:', bestDocDict.title)
// Should output: ['The Old Man and the Sea']

// Assertions
console.assert(searchResults.hits.length === 1, 'Should find one document')
console.assert(bestDocDict.title[0] === 'The Old Man and the Sea', 'Document title should match')
console.assert(bestDocDict.doc_id[0] === 1, 'Document ID should be 1')
console.assert(bestDocDict.body[0].includes('eighty-four days'), 'Body should contain search terms')
```

The `parseQuery` method takes in a query string and creates a `Query` object that can be used to search the index.

In Tantivy, hit documents during search will return a `DocAddress` object that can be used to retrieve the document from the searcher, rather than returning the document directly.

## Building and Executing Queries with Query Objects

> _This is an advanced topic. Only consider this if you need very fine-grained control over your queries, or existing query parsers do not meet your needs._

If you have a Lucene / ElasticSearch background, you might be more comfortable building nested queries programmatically. Also, some queries (e.g. ConstQuery, DisjunctionMaxQuery) are not supported by the query parser due to their complexity in expression.

Consider the following query in ElasticSearch:

```json
{
  "query": {
    "bool": {
      "must": [
        {
          "dis_max": {
            "queries": [
              {
                "match": {
                  "title": {
                    "query": "fish",
                    "boost": 2
                  }
                }
              },
              {
                "match": {
                  "body": {
                    "query": "eighty-four days",
                    "boost": 1.5
                  }
                }
              }
            ],
            "tie_breaker": 0.3
          }
        }
      ]
    }
  }
}
```

It is impossible to express this query using the query parser. Instead, you can build the query programmatically mixing with the query parser:

<!-- example:complex-query source:../examples/complex-query.ts -->

```typescript
import { SchemaBuilder, Index, Document, Query, Occur } from '../index'

// Setup index (from previous examples)
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
writer.commit()
writer.waitMergingThreads()

const complexQuery = Query.booleanQuery([
  {
    occur: Occur.Must,
    query: Query.disjunctionMaxQuery(
      [
        Query.boostQuery(
          // by default, only the query parser will analyze your query string
          index.parseQuery('fish', ['title']),
          2.0,
        ),
        Query.boostQuery(index.parseQuery('eighty-four days', ['body']), 1.5),
      ],
      0.3,
    ),
  },
])

const searcher = index.searcher()
const results = searcher.search(complexQuery, 10)
console.log(`Found ${results.hits.length} results`)

// Assertions
console.assert(complexQuery !== undefined, 'Complex query should be built')
console.assert(results.hits.length === 1, 'Should find one document with complex query')
console.assert(results.hits[0].score !== undefined && results.hits[0].score > 0, 'Result should have a positive score')
```

## Debugging Queries with explain()

When working with search queries, it's often useful to understand why a particular document matched a query and how its score was calculated. The `explain()` method provides detailed information about the scoring process.

<!-- example:query-explanation source:../examples/query-explanation.ts -->

```typescript
import { SchemaBuilder, Index, Document, Query, Occur } from '../index'

// Setup index and complex query (from previous examples)
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
writer.commit()
writer.waitMergingThreads()

const complexQuery = Query.booleanQuery([
  {
    occur: Occur.Must,
    query: Query.disjunctionMaxQuery(
      [
        Query.boostQuery(index.parseQuery('fish', ['title']), 2.0),
        Query.boostQuery(index.parseQuery('eighty-four days', ['body']), 1.5),
      ],
      0.3,
    ),
  },
])

const searcher = index.searcher()

// Search and get the top result
const result = searcher.search(complexQuery, 10)
if (result.hits.length > 0) {
  const hit = result.hits[0]

  // Get an explanation for why this document matched
  const explanation = complexQuery.explain(searcher, hit.docAddress)

  // The explanation provides a JSON representation of the scoring details
  const explanationJson = explanation.toJSON()
  console.log('Score explanation:', explanationJson)

  // Assertions
  console.assert(explanation !== undefined, 'Explanation should be generated')
  console.assert(explanationJson.length > 0, 'Explanation JSON should not be empty')
  console.assert(explanationJson.includes('value'), 'Explanation should contain score value')
}
```

The `toJSON()` method returns a pretty-printed JSON string that shows the final score value, a breakdown of how the score was calculated, details about which query clauses matched, and the contribution of individual terms.

This is particularly useful when debugging why certain documents rank higher than others.

Example output might look like:

```json
{
  "value": 2.5,
  "description": "sum of:",
  "details": [
    {
      "value": 2.0,
      "description": "weight(title:fish) with boost 2.0"
    },
    {
      "value": 0.5,
      "description": "weight(body:days)"
    }
  ]
}
```

## Using the snippet generator

Let's revisit the query `"fish days"` in our [example](#building-and-executing-queries-with-the-query-parser):

<!-- example:snippet-generation source:../examples/snippet-generation.ts -->

```typescript
import { SchemaBuilder, Index, Document, SnippetGenerator } from '../index'

// Setup index (from previous examples)
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
writer.commit()
writer.waitMergingThreads()

index.reload()
const searcher = index.searcher()

const query = index.parseQuery('fish days', ['title', 'body'])
const searchResults = searcher.search(query, 3)
const bestHit = searchResults.hits[0]
const bestDoc = searcher.doc(bestHit.docAddress)
const bestDocDict = bestDoc.toDict() as any

const hitText = bestDocDict.body[0]
console.log('Hit text:', hitText)
// Should output: "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish."

const snippetGenerator = SnippetGenerator.create(searcher, query, schema, 'body')
const snippet = snippetGenerator.snippetFromDoc(bestDoc)

// The snippet object provides the hit ranges. These are the marker
// offsets in the text that match the query.
const highlights = snippet.highlighted()
const firstHighlight = highlights[0]

console.log('First highlight start:', firstHighlight.start) // Should be 93
console.log('First highlight end:', firstHighlight.end) // Should be 97
console.log('Highlighted text:', hitText.slice(firstHighlight.start, firstHighlight.end)) // Should be "days"

// The snippet object can also generate a marked-up HTML snippet:
const htmlSnippet = snippet.toHtml()
console.log('HTML snippet:', htmlSnippet)
// Should output: "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four <b>days</b> now without taking a <b>fish</b>"

// Assertions
console.assert(highlights.length > 0, 'Should have highlights')
console.assert(firstHighlight.start === 93, 'First highlight should start at position 93')
console.assert(firstHighlight.end === 97, 'First highlight should end at position 97')
console.assert(hitText.slice(firstHighlight.start, firstHighlight.end) === 'days', 'Highlighted text should be "days"')
console.assert(htmlSnippet.includes('<b>days</b>'), 'HTML snippet should contain highlighted days')
console.assert(htmlSnippet.includes('<b>fish</b>'), 'HTML snippet should contain highlighted fish')
```

## Create a Custom Tokenizer (Text Analyzer)

Tantivy provides several built-in tokenizers and filters that can be chained together to create new tokenizers (or 'text analyzers') that better fit your needs.

Node-tantivy lets you access these components, assemble them, and register the result with an index.

Let's walk through creating and registering a custom text analyzer to see how everything fits together.

### Example

First, let's create a text analyzer. As explained further down, a text analyzer is a pipeline consisting of one tokenizer and any number of token filters.

<!-- example:custom-analyzer source:../examples/custom-analyzer.ts -->

```typescript
import { TextAnalyzerBuilder, TokenizerStatic, FilterStatic, Index, SchemaBuilder } from '../index'

const myAnalyzer = new TextAnalyzerBuilder(
  // Create a `Tokenizer` instance.
  // It instructs the builder about which type of tokenizer
  // to create internally and with which arguments.
  TokenizerStatic.regex('(?i)([a-z]+)'),
)
  .filter(
    // Create a `Filter` instance.
    // Like `Tokenizer`, this object provides instructions
    // to the builder.
    FilterStatic.lowercase(),
  )
  .filter(
    // Define custom words.
    FilterStatic.customStopword(['www', 'com']),
  )
  // Finally, build a TextAnalyzer
  // chaining all tokenizer > [filter, ...] steps together.
  .build()

// We can check that our new analyzer is working as expected
// by passing some text to its `.analyze()` method.
const tokens = myAnalyzer.analyze('www.this1website1might1exist.com')
console.log('Analyzed tokens:', tokens)
// Will print: ['this', 'website', 'might', 'exist']

// The next step is to register our analyzer with an index.
const schema = new SchemaBuilder().addTextField('content', { tokenizerName: 'custom_analyzer' }).build()

const index = new Index(schema)
index.registerTokenizer('custom_analyzer', myAnalyzer)

// Validate the results
console.assert(Array.isArray(tokens), 'Tokens should be an array')
console.assert(tokens.length > 0, 'Tokens array should not be empty')
console.assert(tokens.includes('this'), 'Tokens should include "this"')
console.assert(tokens.includes('website'), 'Tokens should include "website"')
console.assert(!tokens.includes('www'), 'Tokens should not include stop word "www"')
console.assert(!tokens.includes('com'), 'Tokens should not include stop word "com"')
```

Summary:

1. Use `TextAnalyzerBuilder`, `TokenizerStatic`, and `FilterStatic` to build a `TextAnalyzer`
2. The analyzer's `.analyze()` method lets you use your analyzer as a tokenizer from TypeScript.
3. Refer to your analyzer's name when building the index schema.
4. Use the same name when registering your analyzer on the index.

### On terminology: Tokenizer vs. Text Analyzer

Node-tantivy mimics Tantivy's interface as closely as possible. This includes minor terminological inconsistencies, one of which is how Tantivy distinguishes between 'tokenizers' and 'text analyzers'.

Quite simply, a 'tokenizer' segments text into tokens. A 'text analyzer' is a pipeline consisting of one tokenizer and zero or more token filters. The `TextAnalyzer` is the primary object of interest when talking about how to change Tantivy's tokenization behavior.

Slightly confusingly, though, the `Index` and `SchemaBuilder` interfaces use 'tokenizer' to mean 'text analyzer'.

This inconsistency can be observed in `SchemaBuilder.addTextField`, e.g. --

```typescript
schemaBuilder.addTextField('field', { tokenizerName: '<analyzer name>' })
```

-- and in the name of the `Index.registerTokenizer(...)` method, which actually serves to register a _text analyzer_.
