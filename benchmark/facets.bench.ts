import { bench, describe, beforeAll, afterAll } from 'vitest'
import { SchemaBuilder, Index, Document, Facet } from '../index'
import { mkdtempSync, rmSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'

describe('Facet Operations Performance', () => {
  let tempDir: string
  let schema: any
  let index: any
  let searcher: any

  beforeAll(() => {
    // Setup index with faceted data
    tempDir = mkdtempSync(join(tmpdir(), 'tantivy-facet-bench-'))
    schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body', { stored: true })
      .addFacetField('category')
      .addFacetField('tags')
      .addFacetField('author')
      .addIntegerField('year', { stored: true, indexed: true })
      .build()
    
    index = new Index(schema, tempDir)
    const writer = index.writer()

    // Add documents with various facet combinations
    const categories = ['technology', 'science', 'business', 'entertainment', 'sports']
    const authors = ['alice', 'bob', 'charlie', 'diana', 'eve']
    const tagSets = [
      ['/programming/javascript', '/web/frontend'],
      ['/science/physics', '/research/quantum'],
      ['/business/startup', '/finance/venture'],
      ['/entertainment/movies', '/review/critic'],
      ['/sports/football', '/analysis/statistics']
    ]

    for (let i = 0; i < 5000; i++) {
      const doc = new Document()
      const categoryIndex = i % categories.length
      const authorIndex = i % authors.length
      const tagSetIndex = i % tagSets.length
      
      doc.addText('title', `Document ${i}`)
      doc.addText('body', `Content for document ${i} in category ${categories[categoryIndex]}`)
      doc.addFacet('category', `/category/${categories[categoryIndex]}`)
      doc.addFacet('author', `/author/${authors[authorIndex]}`)
      doc.addInteger('year', 2020 + (i % 5))
      
      // Add multiple tags
      for (const tag of tagSets[tagSetIndex]) {
        doc.addFacet('tags', tag)
      }
      
      writer.addDocument(doc)
    }
    
    writer.commit()
    searcher = index.searcher()
  })

  afterAll(() => {
    try {
      rmSync(tempDir, { recursive: true, force: true })
    } catch (e) {
      // Ignore cleanup errors
    }
  })

  bench('Create simple facet', () => {
    Facet.fromString('/category/technology')
  })

  bench('Create nested facet', () => {
    Facet.fromString('/category/technology/programming/javascript')
  })

  bench('Create 100 facets', () => {
    for (let i = 0; i < 100; i++) {
      Facet.fromString(`/category/tech/subcategory${i % 10}/item${i}`)
    }
  })

  bench('Search with facet filter - single category', () => {
    const query = index.parseQuery('category:/category/technology', [])
    searcher.search(query, 10)
  })

  bench('Search with facet filter - multiple categories', () => {
    const query = index.parseQuery('category:/category/technology OR category:/category/science', [])
    searcher.search(query, 10)
  })

  bench('Search with nested facet filter', () => {
    const query = index.parseQuery('tags:/programming/javascript', [])
    searcher.search(query, 10)
  })

  bench('Complex facet query with text search', () => {
    const query = index.parseQuery('title:document AND category:/category/technology', ['title'])
    searcher.search(query, 10)
  })

  bench('Facet aggregation simulation (multiple searches)', () => {
    const categories = ['technology', 'science', 'business', 'entertainment', 'sports']
    
    // Simulate facet counting by doing multiple searches
    for (const category of categories) {
      const query = index.parseQuery(`category:/category/${category}`, [])
      searcher.search(query, 1) // Just get count, not actual results
    }
  })

  bench('Multi-level facet filtering', () => {
    const query = index.parseQuery('category:/category/technology AND tags:/programming/javascript AND author:/author/alice', [])
    searcher.search(query, 10)
  })

  bench('Facet range combined with text search', () => {
    const query = index.parseQuery('title:document AND year:[2022 TO 2024]', ['title'])
    searcher.search(query, 10)
  })
})
