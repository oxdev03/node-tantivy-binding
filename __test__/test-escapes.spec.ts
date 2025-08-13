import { describe, it, expect, beforeAll } from 'vitest'

import { Document, Index, SchemaBuilder, Query } from '../index'

// Schema builder
const schema = () => {
  return new SchemaBuilder()
    .addTextField('title', { stored: true })
    .addTextField('body')
    .build()
}

// Index creator
const createIndex = (dir?: string) => {
  const index = new Index(schema(), dir)
  const writer = index.writer(15_000_000, 1)

  // Document 1
  const doc1 = new Document()
  doc1.addText('title', 'The Old Man and the Sea')
  doc1.addText(
    'body',
    'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
  )
  writer.addDocument(doc1)

  // Document 2
  const doc2 = Document.fromDict(
    {
      title: 'Of Mice and Men',
      body: "A few miles south of Soledad, the Salinas River drops in close to the hillside bank and runs deep and green. The water is warm too, for it has slipped twinkling over the yellow sands in the sunlight before reaching the narrow pool. On one side of the river the golden foothill slopes curve up to the strong and rocky Gabilan Mountains, but on the valley side the water is lined with trees—willows fresh and green with every spring, carrying in their lower leaf junctures the debris of the winter's flooding; and sycamores with mottled, white, recumbent limbs and branches that arch over the pool",
    },
    schema(),
  )
  writer.addDocument(doc2)

  // Document 3
  writer.addJson(
    JSON.stringify({
      title: ['Frankenstein', 'The Modern Prometheus'],
      body: 'You will rejoice to hear that no disaster has accompanied the commencement of an enterprise which you have regarded with such evil forebodings. I arrived here yesterday, and my first task is to assure my dear sister of my welfare and increasing confidence in the success of my undertaking.',
    }),
  )

  writer.commit()
  writer.waitMergingThreads()
  index.reload()
  return index
}

// Test fixtures
let ramIndex: Index

describe('TestEscapes', () => {
  beforeAll(() => {
    ramIndex = createIndex()
  })

  it('test_escape_quote_parse_query', () => {
    // We verify only that `parseQuery` doesn't raise. This was a change
    // from tantivy versions prior to 0.24.0 in which the following would
    // raise a `ValueError`.
    const query = ramIndex.parseQuery('sea\\"', ['title', 'body'])
    console.log(query.toString())
    expect(query).toBeDefined()
  })

  it('test_escape_quote_parse_query_with_quotes', () => {
    // We verify only that `parseQuery` doesn't raise. We are not testing
    // whether tantivy's `parseQuery` is correct.
    const query = ramIndex.parseQuery('"sea\\""', ['title', 'body'])
    expect(query).toBeDefined()
  })

  it('test_escape_quote_parse_query_quoted', () => {
    // We verify only that `parseQuery` doesn't raise. We are not testing
    // whether tantivy's `parseQuery` is correct.
    const query = ramIndex.parseQuery('title:"sea \\"whale"')
    expect(query).toBeDefined()
  })

  it('test_escape_quote_term_query', () => {
    // We verify only that `termQuery` doesn't raise. We are not testing
    // whether tantivy's `termQuery` is correct.
    const query = Query.termQuery(ramIndex.schema, 'title', 'sea" whale')
    expect(query).toBeDefined()
  })
})
