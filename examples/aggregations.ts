import { SchemaBuilder, Index, Document, Query } from '../index'

// Aggregations run over fast fields, so every aggregated field must be fast
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('category', { stored: true, fast: true, tokenizerName: 'raw' })
schemaBuilder.addFloatField('price', { stored: true, indexed: true, fast: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

for (const [category, price] of [
  ['books', 12.5],
  ['books', 30.0],
  ['music', 9.99],
  ['music', 9.99],
] as [string, number][]) {
  const doc = new Document()
  doc.addText('category', category)
  doc.addFloat('price', price)
  writer.addDocument(doc)
}
writer.commit()
writer.waitMergingThreads()
index.reload()

const searcher = index.searcher()
const all = Query.allQuery()

// Aggregation specs are plain objects, and the result comes back as one too
const result = searcher.aggregate(all, {
  by_category: {
    terms: { field: 'category' },
    aggs: { avg_price: { avg: { field: 'price' } } },
  },
}) as any

for (const bucket of result.by_category.buckets) {
  console.log(`${bucket.key}: ${bucket.doc_count} items, avg ${bucket.avg_price.value}`)
}

// cardinality() is a shorthand for the distinct-value count of one field
const distinctPrices = searcher.cardinality(all, 'price')
console.log('distinct prices:', distinctPrices)

// Assertions
console.assert(result.by_category.buckets.length === 2, 'Should produce one bucket per category')
console.assert(distinctPrices === 3, 'There are three distinct prices')
