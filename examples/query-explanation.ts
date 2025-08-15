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
