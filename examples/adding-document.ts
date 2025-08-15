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
