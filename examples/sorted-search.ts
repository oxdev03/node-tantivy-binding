import { SchemaBuilder, Index, Document, Order } from '../index'

// Both ordering and weighting read from fast fields
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addUnsignedField('views', { stored: true, indexed: true, fast: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

for (const [title, views] of [
  ['tantivy basics', 10],
  ['tantivy advanced', 500],
] as [string, number][]) {
  const doc = new Document()
  doc.addText('title', title)
  doc.addUnsigned('views', views)
  writer.addDocument(doc)
}
writer.commit()
writer.waitMergingThreads()
index.reload()

const searcher = index.searcher()
const query = index.parseQuery('tantivy', ['title'])

// orderByField replaces the relevance score with the field value. Each hit
// then carries `order` instead of `score`, typed after the field.
const byViews = searcher.search(query, 10, true, 'views')
console.log('most viewed first:', byViews.hits[0].order)

// Ascending order, and an offset, are available too
const leastViewed = searcher.search(query, 10, true, 'views', 0, Order.Asc)

// weightByField keeps BM25 relevance but multiplies each score by
// log2(2 + fieldValue), so popular documents float up without ignoring the text
const weighted = searcher.search(query, 10, true, undefined, undefined, undefined, 'views')

// Assertions
console.assert(byViews.hits[0].order === 500, 'Descending order puts the most viewed first')
console.assert(leastViewed.hits[0].order === 10, 'Ascending order puts the least viewed first')
console.assert(weighted.hits[0].score !== undefined, 'Weighted search still produces a score')
console.assert(
  (searcher.doc(weighted.hits[0].docAddress).toDict() as any).title[0] === 'tantivy advanced',
  'The heavier document wins once weighted',
)
