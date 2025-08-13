import { bench, describe, beforeAll, afterAll } from 'vitest'
import { SchemaBuilder, Index, Document, SnippetGenerator } from '../index'
import { mkdtempSync, rmSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'

describe('Snippet Generation Performance', () => {
  let tempDir: string
  let schema: any
  let index: any
  let searcher: any
  let snippetGenerator: any

  beforeAll(() => {
    // Setup index with text content for snippet generation
    tempDir = mkdtempSync(join(tmpdir(), 'tantivy-snippet-bench-'))
    schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body', { stored: true })
      .addTextField('content', { stored: true })
      .build()

    index = new Index(schema, tempDir)
    const writer = index.writer()

    // Add documents with rich text content
    const sampleTexts = [
      'JavaScript is a programming language that is one of the core technologies of the World Wide Web. It enables interactive web pages and is an essential part of web applications.',
      'Python is a high-level programming language known for its simplicity and readability. It is widely used in data science, machine learning, and web development.',
      'Machine learning is a method of data analysis that automates analytical model building. It is a branch of artificial intelligence based on the idea that systems can learn from data.',
      'Data science is an interdisciplinary field that uses scientific methods, processes, algorithms and systems to extract knowledge and insights from noisy, structured and unstructured data.',
      'Web development refers to the work involved in developing a website for the Internet or an intranet. Web development can range from developing simple single static pages to complex applications.',
    ]

    for (let i = 0; i < 1000; i++) {
      const doc = new Document()
      const textIndex = i % sampleTexts.length
      const baseText = sampleTexts[textIndex]

      doc.addText('title', `Document ${i}: ${baseText.split('.')[0]}`)
      doc.addText('body', `${baseText} ${baseText.repeat(2)}`) // Make longer content
      doc.addText('content', baseText.repeat(5)) // Even longer content for snippet testing

      writer.addDocument(doc)
    }

    writer.commit()
    searcher = index.searcher()

    // Create snippet generator
    snippetGenerator = new SnippetGenerator()
  })

  afterAll(() => {
    try {
      rmSync(tempDir, { recursive: true, force: true })
    } catch {
      // Ignore cleanup errors
    }
  })

  bench('Create snippet generator', () => {
    new SnippetGenerator()
  })

  bench('Generate snippets for simple query', () => {
    const query = index.parseQuery('programming', ['title', 'body'])
    const results = searcher.search(query, 10)

    for (const hit of results.hits.slice(0, 5)) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'body')
    }
  })

  bench('Generate snippets for complex query', () => {
    const query = index.parseQuery('programming AND (language OR development)', ['title', 'body'])
    const results = searcher.search(query, 10)

    for (const hit of results.hits.slice(0, 5)) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'body')
    }
  })

  bench('Generate snippets for phrase query', () => {
    const query = index.parseQuery('"machine learning"', ['title', 'body', 'content'])
    const results = searcher.search(query, 10)

    for (const hit of results.hits.slice(0, 5)) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'content')
    }
  })

  bench('Generate multiple snippets per document', () => {
    const query = index.parseQuery('data science development', ['title', 'body', 'content'])
    const results = searcher.search(query, 5)

    for (const hit of results.hits) {
      // Generate snippets for multiple fields
      snippetGenerator.createSnippet(query, searcher, hit.address, 'title')
      snippetGenerator.createSnippet(query, searcher, hit.address, 'body')
      snippetGenerator.createSnippet(query, searcher, hit.address, 'content')
    }
  })

  bench('Generate snippets with long content', () => {
    const query = index.parseQuery('interdisciplinary field systems', ['content'])
    const results = searcher.search(query, 10)

    // Generate snippets from the longest field
    for (const hit of results.hits.slice(0, 5)) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'content')
    }
  })

  bench('Batch snippet generation (20 documents)', () => {
    const query = index.parseQuery('programming language web', ['title', 'body'])
    const results = searcher.search(query, 20)

    // Generate snippets for all results
    for (const hit of results.hits) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'body')
    }
  })

  bench('Snippet generation with wildcard query', () => {
    const query = index.parseQuery('develop* program*', ['title', 'body'])
    const results = searcher.search(query, 10)

    for (const hit of results.hits.slice(0, 5)) {
      snippetGenerator.createSnippet(query, searcher, hit.address, 'body')
    }
  })
})
