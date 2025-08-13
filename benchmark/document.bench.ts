import { bench, describe } from 'vitest'
import { Document } from '../index'

describe('Document Operations Performance', () => {
  bench('Create simple document', () => {
    const doc = new Document()
    doc.addText('title', 'Sample Document')
    doc.addText('body', 'This is a sample document for benchmarking.')
    doc.addInteger('id', 1)
  })

  bench('Create document with many fields', () => {
    const doc = new Document()
    doc.addText('title', 'Complex Document with Many Fields')
    doc.addText('body', 'This document has many different types of fields for comprehensive benchmarking. '.repeat(10))
    doc.addText('category', 'benchmark')
    doc.addInteger('id', 12345)
    doc.addFloat('score', 98.7)
    doc.addDate('created_at', '2024-01-15T10:30:00Z')
    doc.addFacet('tags', '/category/benchmark')
    doc.addFacet('tags', '/type/performance')
    doc.addFacet('tags', '/language/typescript')
  })

  bench('Create 100 simple documents', () => {
    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('title', `Document ${i}`)
      doc.addText('body', `Content for document ${i}`)
      doc.addInteger('id', i)
    }
  })

  bench('Create document with large text content', () => {
    const largeText = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. '.repeat(1000) // ~56KB of text
    const doc = new Document()
    doc.addText('title', 'Large Document')
    doc.addText('body', largeText)
    doc.addInteger('id', 1)
  })

  bench('Document with many facets', () => {
    const doc = new Document()
    doc.addText('title', 'Document with Many Facets')
    doc.addText('body', 'This document tests facet performance.')
    doc.addInteger('id', 1)
    
    // Add 50 facets
    for (let i = 0; i < 50; i++) {
      doc.addFacet('tags', `/category${i % 10}/subcategory${i % 5}/item${i}`)
    }
  })

  bench('Document serialization patterns', () => {
    const doc = new Document()
    doc.addText('title', 'Serialization Test Document')
    doc.addText('body', 'Testing various serialization patterns and performance characteristics.')
    doc.addText('category', 'performance')
    doc.addInteger('id', 42)
    doc.addFloat('score', 95.5)
    doc.addDate('created_at', '2024-01-15T10:30:00Z')
    
    // Test data conversion overhead
    JSON.stringify({
      title: 'Serialization Test Document',
      body: 'Testing various serialization patterns and performance characteristics.',
      category: 'performance',
      id: 42,
      score: 95.5,
      created_at: '2024-01-15T10:30:00Z',
      tags: ['/test/serialization', '/type/benchmark']
    })
  })

  bench('Document field access patterns', () => {
    const doc = new Document()
    
    // Test different ways of adding the same type of field
    const fieldNames = ['field1', 'field2', 'field3', 'field4', 'field5']
    const values = ['value1', 'value2', 'value3', 'value4', 'value5']
    
    for (let i = 0; i < fieldNames.length; i++) {
      doc.addText(fieldNames[i], values[i])
    }
  })
})
