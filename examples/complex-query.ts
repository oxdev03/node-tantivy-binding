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
