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
