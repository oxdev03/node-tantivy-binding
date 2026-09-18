import { SchemaBuilder, Index, Document, Query } from '../index'

// termsWithPrefix walks the term dictionary, so the field only needs indexing.
// fastFieldValues reads a column, so that field must be declared fast.
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('body')
schemaBuilder.addUnsignedField('owner_id', { stored: true, indexed: true, fast: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

for (const [body, owner] of [
  ['apple banana', 1],
  ['apple apricot', 2],
  ['cherry date', 1],
] as [string, number][]) {
  const doc = new Document()
  doc.addText('body', body)
  doc.addUnsigned('owner_id', owner)
  writer.addDocument(doc)
}
writer.commit()
writer.waitMergingThreads()
index.reload()

const searcher = index.searcher()

// Suggestions for a prefix, sorted by document frequency then alphabetically
const suggestions = searcher.termsWithPrefix('body', 'ap')
for (const { term, count } of suggestions) {
  console.log(`${term} (${count})`)
}

// A filter query scopes the counts, e.g. to documents the user may see
const ownedByUser2 = Query.termQuery(schema, 'owner_id', 2)
const scoped = searcher.termsWithPrefix('body', 'ap', ownedByUser2, 5)

// Reading one numeric column for many hits is far cheaper than fetching
// each stored document just to pull a single field out of it
const hits = searcher.search(index.parseQuery('apple', ['body']), 10).hits
const owners = searcher.fastFieldValues(
  'owner_id',
  hits.map((hit) => hit.docAddress),
)
console.log('owners of matching documents:', owners)

// Assertions
console.assert(suggestions[0].term === 'apple' && suggestions[0].count === 2, 'apple appears in two documents')
console.assert(scoped.length === 2, 'user 2 sees both apple and apricot')
console.assert(owners.length === 2, 'One value per hit, in hit order')
