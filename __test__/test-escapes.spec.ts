import { describe, it, expect, beforeAll } from 'vitest'

import { Index, Query } from '../index'
import { createIndex } from './fixtures'

describe('TestEscapes', () => {
  let ramIndex: Index

  beforeAll(() => {
    ramIndex = createIndex()
  })

  it('test_escape_quote_parse_query', () => {
    // We verify only that `parseQuery` doesn't raise. This was a change
    // from tantivy versions prior to 0.24.0 in which the following would
    // raise a `ValueError`.
    const query = ramIndex.parseQuery('sea\\"', ['title', 'body'])
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
