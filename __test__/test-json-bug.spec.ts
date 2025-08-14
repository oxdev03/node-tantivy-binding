import { describe, it, expect } from 'vitest'

import { Document, Index, SchemaBuilder } from '../index'

describe('TestJsonBug', () => {
  it('test_json_bug', () => {
    const schemaBuilder = new SchemaBuilder()
    schemaBuilder.addJsonField('data', { stored: true })
    const schema = schemaBuilder.build()

    const index = new Index(schema)
    const indexWriter = index.writer()

    const data = {
      name: 'John Doe',
      age: 30,
      email: 'john.doe@example.com',
      interests: ['reading', 'hiking', 'coding'],
    }

    const doc = new Document()
    doc.addJson('data', data)
    indexWriter.addDocument(doc)
    indexWriter.commit()
    indexWriter.waitMergingThreads()
    index.reload()

    const searcher = index.searcher()

    const query = '*'
    const q = index.parseQuery(query)
    const topDocs = searcher.search(q, 10)

    expect(topDocs.hits.length).toBeGreaterThan(0)

    for (const hit of topDocs.hits) {
      const doc = searcher.doc(hit.docAddress)
      const docDict = doc.toDict() as any

      // With array support added, JSON fields should be stored as arrays like other fields
      expect(docDict.data).toEqual([
        {
          age: 30,
          email: 'john.doe@example.com',
          interests: ['reading', 'hiking', 'coding'],
          name: 'John Doe',
        },
      ])
    }
  })
})
