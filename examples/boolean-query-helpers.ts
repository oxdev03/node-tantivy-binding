import { SchemaBuilder, Index, Document, Query } from '../index'

// Setup index with three books
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

for (const [title, body] of [
  ['The Old Man and the Sea', 'He was an old man who fished alone in a skiff in the Gulf Stream.'],
  ['Of Mice and Men', 'A few miles south of Soledad, the Salinas River drops in close to the hillside bank.'],
  ['Frankenstein', 'You will rejoice to hear that no disaster has accompanied the commencement.'],
]) {
  const doc = new Document()
  doc.addText('title', title)
  doc.addText('body', body)
  writer.addDocument(doc)
}
writer.commit()
writer.waitMergingThreads()
index.reload()

const searcher = index.searcher()
const old = Query.termQuery(schema, 'title', 'old')
const man = Query.termQuery(schema, 'title', 'man')
const mice = Query.termQuery(schema, 'title', 'mice')

// AND: both terms must appear in the title
const both = old.andMustMatch([man])
console.log('old AND man:', searcher.search(both, 10).hits.length)

// OR: either query may match. Several queries can be passed at once
const either = old.orShouldMatch([mice])
console.log('old OR mice:', searcher.search(either, 10).hits.length)

// AND NOT: exclude documents matching the given queries
const withoutSea = Query.termQuery(schema, 'body', 'the').andMustNotMatch([old])
console.log('the AND NOT old:', searcher.search(withoutSea, 10).hits.length)

// Chains stay flat, and mixing AND with OR groups the left-hand side:
// (old AND man) OR mice
const mixed = old.andMustMatch([man]).orShouldMatch([mice])

// Assertions
console.assert(searcher.search(both, 10).hits.length === 1, 'old AND man should match one document')
console.assert(searcher.search(either, 10).hits.length === 2, 'old OR mice should match two documents')
console.assert(searcher.search(withoutSea, 10).hits.length === 2, 'excluding "old" should leave two documents')
console.assert(searcher.search(mixed, 10).hits.length === 2, '(old AND man) OR mice should match two documents')
