import { bench, describe, beforeEach, afterEach } from 'vitest'
import { SchemaBuilder, Index, Document } from '../index'
import { mkdtempSync, rmSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'

describe('Indexing Performance', () => {
  let tempDir: string
  let schema: any
  let index: any

  beforeEach(() => {
    tempDir = mkdtempSync(join(tmpdir(), 'tantivy-bench-'))
    schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body', { stored: true })
      .addIntegerField('id', { stored: true, indexed: true })
      .build()
    index = new Index(schema, tempDir)
  })

  afterEach(() => {
    try {
      rmSync(tempDir, { recursive: true, force: true })
    } catch {
      // Ignore cleanup errors
    }
  })

  bench('Index 100 small documents', () => {
    const writer = index.writer()
    
    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('title', `Document ${i}`)
      doc.addText('body', `This is the body of document ${i}. It contains some sample text for indexing.`)
      doc.addInteger('id', i)
      writer.addDocument(doc)
    }
    
    writer.commit()
  })

  bench('Index 1000 small documents', () => {
    const writer = index.writer()
    
    for (let i = 0; i < 1000; i++) {
      const doc = new Document()
      doc.addText('title', `Document ${i}`)
      doc.addText('body', `This is the body of document ${i}. It contains some sample text for indexing.`)
      doc.addInteger('id', i)
      writer.addDocument(doc)
    }
    
    writer.commit()
  })

  bench('Index 100 large documents', () => {
    const writer = index.writer()
    const largeText = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. '.repeat(100)
    
    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('title', `Large Document ${i}`)
      doc.addText('body', `${largeText} Document ${i} specific content.`)
      doc.addInteger('id', i)
      writer.addDocument(doc)
    }
    
    writer.commit()
  })

  bench('Batch commit vs individual commits (100 docs)', () => {
    const writer = index.writer()
    
    // Batch commit
    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('title', `Batch Document ${i}`)
      doc.addText('body', `Batch document ${i} content for performance testing.`)
      doc.addInteger('id', i)
      writer.addDocument(doc)
    }
    writer.commit()
  })
})
