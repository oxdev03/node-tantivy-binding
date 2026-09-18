import { describe, it, expect } from 'vitest'

import { Document, Index, SchemaBuilder } from '../index'

const scoringSchema = () =>
  new SchemaBuilder()
    .addIntegerField('id', { stored: true, indexed: true, fast: true })
    .addFloatField('weight_f64', { stored: true, indexed: true, fast: true })
    .addIntegerField('weight_i64', { stored: true, indexed: true, fast: true })
    .addUnsignedField('weight_u64', { stored: true, indexed: true, fast: true })
    .addTextField('body', { stored: true, fast: true })
    .build()

describe('TestDocumentScoring', () => {
  for (const weightByField of ['weight_f64', 'weight_i64', 'weight_u64']) {
    it(`test_document_scoring_${weightByField}`, () => {
      const index = new Index(scoringSchema())
      const writer = index.writer(15_000_000, 1)

      const doc1 = new Document()
      doc1.addInteger('id', 1)
      doc1.addFloat('weight_f64', 0.1)
      doc1.addInteger('weight_i64', 1)
      doc1.addUnsigned('weight_u64', 1)
      doc1.addText('body', 'apple banana orange mango')
      writer.addDocument(doc1)

      const doc2 = new Document()
      doc2.addInteger('id', 2)
      doc2.addFloat('weight_f64', 0.9)
      doc2.addInteger('weight_i64', 10)
      doc2.addUnsigned('weight_u64', 10)
      doc2.addText('body', 'pear lemon tomato banana')
      writer.addDocument(doc2)

      writer.commit()
      writer.waitMergingThreads()
      index.reload()

      const searcher = index.searcher()
      const query = index.parseQuery('body:banana')

      // Without weighting, the shorter-field document wins on BM25.
      let results = searcher.search(query, 1)
      expect(results.hits.length).toBe(1)
      expect((searcher.doc(results.hits[0].docAddress).toDict() as any).id).toEqual([1])

      // Weighting by a fast field promotes the heavier document.
      results = searcher.search(query, 1, true, undefined, undefined, undefined, weightByField)
      expect(results.hits.length).toBe(1)
      expect((searcher.doc(results.hits[0].docAddress).toDict() as any).id).toEqual([2])
    })
  }

  it('test_not_fastfield', () => {
    const index = new Index(
      new SchemaBuilder()
        .addIntegerField('id', { stored: true, indexed: true, fast: true })
        .addFloatField('weight_f64', { stored: true, indexed: true, fast: false })
        .addTextField('body', { stored: true, fast: true })
        .build(),
    )
    index.reload()

    const searcher = index.searcher()
    const query = index.parseQuery('body:banana')
    expect(() => searcher.search(query, 1, true, undefined, undefined, undefined, 'weight_f64')).toThrow(
      'not a fast field',
    )
  })
})
