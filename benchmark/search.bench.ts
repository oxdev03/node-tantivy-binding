import { bench, describe, beforeAll, afterAll } from 'vitest'
import { SchemaBuilder, Index, Document } from '../index'
import { mkdtempSync, rmSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'

describe('Search Performance', () => {
  let tempDir: string
  let schema: any
  let index: any
  let searcher: any

  beforeAll(() => {
    // Setup index with test data
    tempDir = mkdtempSync(join(tmpdir(), 'tantivy-search-bench-'))
    schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body', { stored: true })
      .addTextField('category', { stored: true })
      .addIntegerField('id', { stored: true, indexed: true })
      .addIntegerField('score', { stored: true, indexed: true })
      .build()

    index = new Index(schema, tempDir)
    const writer = index.writer()

    // Add 10,000 documents for realistic search benchmarks
    const categories = ['tech', 'science', 'business', 'sports', 'entertainment']
    const titles = [
      'Introduction to Machine Learning',
      'Advanced Database Systems',
      'Web Development Best Practices',
      'Data Science Fundamentals',
      'Software Engineering Principles',
      'Mobile App Development',
      'Cloud Computing Essentials',
      'Cybersecurity Basics',
      'Artificial Intelligence Overview',
      'Blockchain Technology',
    ]

    for (let i = 0; i < 10000; i++) {
      const doc = new Document()
      const titleIndex = i % titles.length
      const categoryIndex = i % categories.length

      doc.addText('title', `${titles[titleIndex]} ${i}`)
      doc.addText(
        'body',
        `This is document ${i} about ${titles[titleIndex].toLowerCase()}. It contains detailed information and examples. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.`,
      )
      doc.addText('category', categories[categoryIndex])
      doc.addInteger('id', i)
      doc.addInteger('score', Math.floor(Math.random() * 100))
      writer.addDocument(doc)
    }

    writer.commit()
    searcher = index.searcher()
  })

  afterAll(() => {
    try {
      rmSync(tempDir, { recursive: true, force: true })
    } catch {
      // Ignore cleanup errors
    }
  })

  bench('Simple term search', () => {
    const query = index.parseQuery('machine', ['title', 'body'])
    searcher.search(query, 10)
  })

  bench('Complex boolean search', () => {
    const query = index.parseQuery('(machine OR database) AND development', ['title', 'body'])
    searcher.search(query, 10)
  })

  bench('Phrase search', () => {
    const query = index.parseQuery('"machine learning"', ['title', 'body'])
    searcher.search(query, 10)
  })

  bench('Wildcard search', () => {
    const query = index.parseQuery('develop*', ['title', 'body'])
    searcher.search(query, 10)
  })

  bench('Range query on integer field', () => {
    const query = index.parseQuery('id:[1000 TO 2000]', ['id'])
    searcher.search(query, 10)
  })

  bench('Search with large result set (100 results)', () => {
    const query = index.parseQuery('the', ['title', 'body'])
    searcher.search(query, 100)
  })

  bench('Search with very large result set (1000 results)', () => {
    const query = index.parseQuery('the', ['title', 'body'])
    searcher.search(query, 1000)
  })

  bench('Multiple field search', () => {
    const query = index.parseQuery('development', ['title', 'body', 'category'])
    searcher.search(query, 10)
  })

  bench('Search with explain (debugging overhead)', () => {
    const query = index.parseQuery('machine learning', ['title', 'body'])
    const results = searcher.search(query, 5)

    // Explain top results
    for (const hit of results.hits.slice(0, 3)) {
      query.explain(searcher, hit.address)
    }
  })
})
