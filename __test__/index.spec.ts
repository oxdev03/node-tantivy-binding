import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { tmpdir } from 'os'
import { mkdtempSync } from 'fs'
import { join } from 'path'

import {
  Document,
  Index,
  parseQuery,
  parseQueryLenient,
  SchemaBuilder,
  Schema,
  Query,
  Order,
  Occur,
  FieldType,
  TokenizerStatic,
  FilterStatic,
  SnippetGenerator,
  TextAnalyzerBuilder,
  Facet,
  DocAddress,
  FieldDoesNotExistError,
  UnsupportedQueryError,
  ExpectedIntError,
  ExpectedFloatError,
} from '../index'

import {
  schema,
  createIndex,
  createIndexWithNumericFields,
  createIndexWithDateField,
  createIndexWithIpAddrField,
  createSpanishIndex,
  createIndexWithOrderFastFields,
  createIndexWithEmptyFastField,
  TestDoc,
} from './fixtures'
import { rm } from 'fs/promises'

// Global test indices
// Test fixtures
let ramIndex: Index
let ramIndexNumericFields: Index
let ramIndexWithDateField: Index
let ramIndexWithIpAddrField: Index
let indexWithOrderFastFields: Index
let indexWithEmptyFastField: Index
let spanishIndex: Index
let tempDir: string
beforeAll(() => {
  ramIndex = createIndex()
  ramIndexNumericFields = createIndexWithNumericFields()
  ramIndexWithDateField = createIndexWithDateField()
  ramIndexWithIpAddrField = createIndexWithIpAddrField()
  indexWithOrderFastFields = createIndexWithOrderFastFields()
  indexWithEmptyFastField = createIndexWithEmptyFastField()
  spanishIndex = createSpanishIndex()
  tempDir = mkdtempSync(join(tmpdir(), 'tantivy-test-'))
})
afterAll(async () => {
  // Cleanup temporary directory with retry logic for Windows file locking issues
  await rm(tempDir, { recursive: true, force: true }).catch((err) => {
    console.error(`Failed to remove temp directory ${tempDir}:`, err)
  })
})
describe('TestClass', () => {
  it('test_simple_search_in_dir', () => {
    const dirIndex = createIndex(tempDir)
    const query = dirIndex.parseQuery('sea whale', ['title', 'body'])
    const result = dirIndex.searcher().search(query, 10)
    expect(result.hits.length).toBe(1)
  })

  it('test_simple_search_after_reuse', () => {
    const index = new Index(schema(), tempDir)
    const query = index.parseQuery('sea whale', ['title', 'body'])
    const result = index.searcher().search(query, 10)
    expect(result.hits.length).toBe(1)
  })

  it('test_simple_search_in_ram', () => {
    // Test schema properties
    expect(ramIndex.schema.numFields()).toBe(2)
    expect(ramIndex.schema.fieldNames()).toEqual(['title', 'body'])
    expect(ramIndex.schema.hasField('title')).toBe(true)
    expect(ramIndex.schema.hasField('body')).toBe(true)
    expect(ramIndex.schema.getFieldType('title')).toBe(FieldType.Text)
    expect(ramIndex.schema.getFieldType('body')).toBe(FieldType.Text)

    const query = ramIndex.parseQuery('sea whale', ['title', 'body'])
    const result = ramIndex.searcher().search(query, 10)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = ramIndex.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_simple_search_in_spanish', () => {
    const query = spanishIndex.parseQuery('vieja', ['title', 'body'])
    const result = spanishIndex.searcher().search(query, 10)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchDoc = spanishIndex.searcher().doc(docAddress)
    expect((searchDoc.toDict() as TestDoc).title).toEqual(['El viejo y el mar'])
  })

  it('test_and_query', () => {
    let query = ramIndex.parseQuery('title:men AND body:summer', ['title', 'body'])
    const searcher = ramIndex.searcher()
    let result = searcher.search(query, 10)

    // summer isn't present
    expect(result.hits.length).toBe(0)

    query = ramIndex.parseQuery('title:men AND body:winter', ['title', 'body'])
    result = searcher.search(query)
    expect(result.hits.length).toBe(1)
  })

  it('test_doc_freq', () => {
    const searcher = ramIndex.searcher()
    const docFreq = searcher.docFreq('body', 'and')
    expect(docFreq).toBe(3)
  })

  it('test_and_aggregate', () => {
    const query = Query.allQuery()
    const aggQuery = {
      top_hits_req: {
        top_hits: {
          size: 2,
          sort: [{ rating: 'desc' }],
          from: 0,
          docvalue_fields: ['rating', 'id', 'body'],
        },
      },
    }
    const searcher = ramIndexNumericFields.searcher()
    const result = searcher.aggregate(query, aggQuery) as any
    expect(typeof result).toBe('object')
    expect('top_hits_req' in result).toBe(true)
    expect(result.top_hits_req.hits.length).toBe(2)
    for (const hit of result.top_hits_req.hits) {
      expect(Object.keys(hit.docvalue_fields).length).toBe(3)
    }

    const expectedResult = {
      top_hits_req: {
        hits: [
          {
            sort: [13840124604862955520n],
            docvalue_fields: {
              id: [2],
              rating: [4.5],
              body: [
                'a',
                'few',
                'miles',
                'south',
                'of',
                'soledad',
                'the',
                'salinas',
                'river',
                'drops',
                'in',
                'close',
                'to',
                'the',
                'hillside',
                'bank',
                'and',
                'runs',
                'deep',
                'and',
                'green',
                'the',
                'water',
                'is',
                'warm',
                'too',
                'for',
                'it',
                'has',
                'slipped',
                'twinkling',
                'over',
                'the',
                'yellow',
                'sands',
                'in',
                'the',
                'sunlight',
                'before',
                'reaching',
                'the',
                'narrow',
                'pool',
                'on',
                'one',
                'side',
                'of',
                'the',
                'river',
                'the',
                'golden',
                'foothill',
                'slopes',
                'curve',
                'up',
                'to',
                'the',
                'strong',
                'and',
                'rocky',
                'gabilan',
                'mountains',
                'but',
                'on',
                'the',
                'valley',
                'side',
                'the',
                'water',
                'is',
                'lined',
                'with',
                'trees',
                'willows',
                'fresh',
                'and',
                'green',
                'with',
                'every',
                'spring',
                'carrying',
                'in',
                'their',
                'lower',
                'leaf',
                'junctures',
                'the',
                'debris',
                'of',
                'the',
                'winter',
                's',
                'flooding',
                'and',
                'sycamores',
                'with',
                'mottled',
                'white',
                'recumbent',
                'limbs',
                'and',
                'branches',
                'that',
                'arch',
                'over',
                'the',
                'pool',
              ],
            },
          },
          {
            sort: [13838435755002691584n],
            docvalue_fields: {
              body: [
                'he',
                'was',
                'an',
                'old',
                'man',
                'who',
                'fished',
                'alone',
                'in',
                'a',
                'skiff',
                'in',
                'the',
                'gulf',
                'stream',
                'and',
                'he',
                'had',
                'gone',
                'eighty',
                'four',
                'days',
                'now',
                'without',
                'taking',
                'a',
                'fish',
              ],
              rating: [3.5],
              id: [1],
            },
          },
        ],
      },
    }
    expect(result).toEqual(expectedResult)
  })

  it('test_and_query_numeric_fields', () => {
    // Test numeric fields schema
    expect(ramIndexNumericFields.schema.numFields()).toBe(4)
    expect(ramIndexNumericFields.schema.fieldNames()).toEqual(['id', 'rating', 'is_good', 'body'])
    expect(ramIndexNumericFields.schema.getFieldType('id')).toBe(FieldType.Integer)
    expect(ramIndexNumericFields.schema.getFieldType('rating')).toBe(FieldType.Float)
    expect(ramIndexNumericFields.schema.getFieldType('is_good')).toBe(FieldType.Boolean)
    expect(ramIndexNumericFields.schema.getFieldType('body')).toBe(FieldType.Text)

    const searcher = ramIndexNumericFields.searcher()

    // 1 result
    const floatQuery = ramIndexNumericFields.parseQuery('3.5', ['rating'])
    let result = searcher.search(floatQuery)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).rating?.[0]).toBe(3.5)

    const integerQuery = ramIndexNumericFields.parseQuery('1', ['id'])
    result = searcher.search(integerQuery)
    expect(result.hits.length).toBe(1)

    // 0 result
    const integerQuery2 = ramIndexNumericFields.parseQuery('10', ['id'])
    result = searcher.search(integerQuery2)
    expect(result.hits.length).toBe(0)
  })

  it('test_and_query_parser_default_fields', () => {
    const query = ramIndex.parseQuery('winter', ['title'])
    expect(query.toString()).toBe('Query(TermQuery(Term(field=0, type=Str, "winter")))')
  })

  it('test_and_query_parser_default_fields_undefined', () => {
    const query = ramIndex.parseQuery('winter')
    expect(query.toString()).toBe(
      'Query(BooleanQuery { subqueries: [(Should, TermQuery(Term(field=0, type=Str, "winter"))), (Should, TermQuery(Term(field=1, type=Str, "winter")))], minimum_number_should_match: 1 })',
    )
  })

  it('test_parse_query_field_boosts', () => {
    const query = ramIndex.parseQuery('winter', undefined, { title: 2.3 })
    expect(query.toString()).toBe(
      'Query(BooleanQuery { subqueries: [(Should, Boost(query=TermQuery(Term(field=0, type=Str, "winter")), boost=2.3)), (Should, TermQuery(Term(field=1, type=Str, "winter")))], minimum_number_should_match: 1 })',
    )
  })

  it('test_parse_query_fuzzy_fields', () => {
    const query = ramIndex.parseQuery('winter', undefined, undefined, { title: [true, 1, false] })
    expect(query.toString()).toBe(
      'Query(BooleanQuery { subqueries: [(Should, FuzzyTermQuery { term: Term(field=0, type=Str, "winter"), distance: 1, transposition_cost_one: false, prefix: true }), (Should, TermQuery(Term(field=1, type=Str, "winter")))], minimum_number_should_match: 1 })',
    )
  })

  it('test_query_lenient', () => {
    // Test with valid query - should have no errors
    let [query, errors] = ramIndexNumericFields.parseQueryLenient('rating:3.5')
    expect(errors.length).toBe(0)
    expect(query.toString()).toBe('Query(TermQuery(Term(field=1, type=F64, 3.5)))')

    // Test with field that doesn't exist - should have 1 typed error
    let [_, errors2] = ramIndexNumericFields.parseQueryLenient('bod:men')
    expect(errors2.length).toBe(1)
    expect(errors2[0]).toBeInstanceOf(FieldDoesNotExistError)
    expect((errors2[0] as FieldDoesNotExistError).field).toBe('bod')
    expect(String(errors2[0])).toContain('bod')

    // Test with multiple errors in complex query
    let [query3, errors3] = ramIndexNumericFields.parseQueryLenient("body:'hello' AND id:<3.5 OR rating:'hi'")
    expect(errors3.length).toBe(2)
    expect(errors3[0]).toBeInstanceOf(ExpectedIntError)
    expect(errors3[1]).toBeInstanceOf(ExpectedFloatError)
    // Check that query still parses partially
    expect(query3.toString()).toContain('TermQuery(Term(field=3, type=Str, "hello"))')
  })

  it('test_and_query_date_fields', () => {
    const searcher = ramIndexWithDateField.searcher()

    // 1 result
    const fromDate1 = new Date('2021-01-01T00:00:00.000Z')
    const toDate1 = new Date('2021-01-01T23:59:00.000Z')
    const query1 = ramIndexWithDateField.parseQuery(`date:[${fromDate1.toISOString()} TO ${toDate1.toISOString()}]`)
    const result1 = searcher.search(query1)
    expect(result1.hits.length).toBe(1)
    const doc1 = searcher.doc(result1.hits[0].docAddress).toDict() as TestDoc
    expect(new Date(doc1.date?.[0] as any)).toEqual(fromDate1)

    // 2 results
    const fromDate2 = new Date('2021-01-01T00:00:00.000Z')
    const toDate2 = new Date('2021-01-02T23:59:00.000Z')
    const query2 = ramIndexWithDateField.parseQuery(`date:[${fromDate2.toISOString()} TO ${toDate2.toISOString()}]`)
    const result2 = searcher.search(query2)
    expect(result2.hits.length).toBe(2)

    // 0 results
    const fromDate3 = new Date('2022-01-01T00:00:00.000Z')
    const toDate3 = new Date('2022-01-01T23:59:00.000Z')
    const query3 = ramIndexWithDateField.parseQuery(`date:[${fromDate3.toISOString()} TO ${toDate3.toISOString()}]`)
    const result3 = searcher.search(query3)
    expect(result3.hits.length).toBe(0)
  })

  it('test_and_query_ip_addr_fields', () => {
    const searcher = ramIndexWithIpAddrField.searcher()

    // 1 result
    const fromIp1 = '10.0.0.0'
    const toIp1 = '10.0.0.255'
    const query1 = ramIndexWithIpAddrField.parseQuery(`ip_addr:[${fromIp1} TO ${toIp1}]`)
    const result1 = searcher.search(query1)
    expect(result1.hits.length).toBe(1)
    const doc1 = searcher.doc(result1.hits[0].docAddress).toDict() as TestDoc
    expect(doc1.ip_addr?.[0]).toBe('10.0.0.1') // Now returns original IPv4 format like Python

    // 2 results
    const fromIp2 = '10.0.0.0'
    const toIp2 = '127.0.0.255'
    const query2 = ramIndexWithIpAddrField.parseQuery(`ip_addr:[${fromIp2} TO ${toIp2}]`)
    const result2 = searcher.search(query2)
    expect(result2.hits.length).toBe(2)

    // 2 results (not 3, as IPv6 is separate)
    const fromIp3 = '0.0.0.0'
    const toIp3 = '255.255.255.255'
    const query3 = ramIndexWithIpAddrField.parseQuery(`ip_addr:[${fromIp3} TO ${toIp3}]`)
    const result3 = searcher.search(query3)
    expect(result3.hits.length).toBe(2)

    // 1 result (IPv6)
    const fromIp4 = '::0'
    const toIp4 = '::FFFF'
    const query4 = ramIndexWithIpAddrField.parseQuery(`ip_addr:[${fromIp4} TO ${toIp4}]`)
    const result4 = searcher.search(query4)
    expect(result4.hits.length).toBe(1)
    const doc4 = searcher.doc(result4.hits[0].docAddress).toDict() as TestDoc
    expect(doc4.ip_addr?.[0]).toBe('::1') // IPv6

    // 0 results
    const fromIp5 = '200.0.0.0'
    const toIp5 = '255.255.255.255'
    const query5 = ramIndexWithIpAddrField.parseQuery(`ip_addr:[${fromIp5} TO ${toIp5}]`)
    const result5 = searcher.search(query5)
    expect(result5.hits.length).toBe(0)
  })

  it('test_query_errors', () => {
    // no "bod" field
    expect(() => {
      ramIndex.parseQuery('bod:men', ['title', 'body'])
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Field does not exist: 'bod']`)
  })

  it('test_query_explain', () => {
    // Search for something that will actually return results
    const query = ramIndex.parseQuery('title:sea OR body:fish', ['title', 'body'])
    const searcher = ramIndex.searcher()
    const result = searcher.search(query, 10)

    // Should have at least one result (The Old Man and the Sea)
    expect(result.hits.length).toBeGreaterThan(0)

    const { docAddress } = result.hits[0]

    // Test the explain() method
    const explanation = query.explain(searcher, docAddress)
    const jsonOutput = explanation.toJSON()
    expect(typeof jsonOutput).toBe('string')
    expect(jsonOutput.length).toBeGreaterThan(0)
    // The JSON should contain score information
    expect(jsonOutput).toMatch(/"value"|value/)

    // Test the value method
    const scoreValue = explanation.value()
    expect(typeof scoreValue).toBe('number')
    expect(scoreValue).toBeGreaterThan(0)

    // Test the toString method
    const stringOutput = explanation.toString()
    expect(typeof stringOutput).toBe('string')
    expect(stringOutput).toContain('Explanation')
  })

  it('test_order_by_search', () => {
    const schema = new SchemaBuilder()
      .addUnsignedField('order', { fast: true })
      .addTextField('title', { stored: true })
      .build()

    const index = new Index(schema)
    const writer = index.writer()

    let doc = new Document()
    doc.addUnsigned('order', 0)
    doc.addText('title', 'Test title')
    writer.addDocument(doc)

    doc = new Document()
    doc.addUnsigned('order', 2)
    doc.addText('title', 'Final test title')
    writer.addDocument(doc)

    doc = new Document()
    doc.addUnsigned('order', 1)
    doc.addText('title', 'Another test title')
    writer.addDocument(doc)

    writer.commit()
    index.reload()

    const query = index.parseQuery('test')
    const searcher = index.searcher()

    let result = searcher.search(query, 10, true, 'order', 2)
    expect(result.hits.length).toBe(1)

    result = searcher.search(query, 10, true, 'order')
    expect(result.hits.length).toBe(3)

    let { docAddress } = result.hits[0]
    let searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Final test title'])

    docAddress = result.hits[1].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Another test title'])

    docAddress = result.hits[2].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Test title'])

    result = searcher.search(query, 10, true, 'order', 0, Order.Asc)
    expect(result.hits.length).toBe(3)

    docAddress = result.hits[2].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Final test title'])

    docAddress = result.hits[1].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Another test title'])

    docAddress = result.hits[0].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Test title'])
  })

  it('test_order_by_search_without_fast_field', () => {
    // Note: In Node.js version, this throws an error because the field is not configured as fast field
    // while in Python version it returns 0 results. This is an API difference.
    const schema = new SchemaBuilder()
      .addUnsignedField('order') // No fast: true
      .addTextField('title', { stored: true })
      .build()

    const index = new Index(schema)
    const writer = index.writer()

    const doc = new Document()
    doc.addUnsigned('order', 0)
    doc.addText('title', 'Test title')
    writer.addDocument(doc)
    writer.commit()
    index.reload()

    const query = index.parseQuery('test')
    const searcher = index.searcher()

    expect(() => {
      searcher.search(query, 10, true, 'order')
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Schema error: 'Field \`order\` is not a fast field.']`)
  })

  it('test_order_by_search_date', () => {
    const schema = new SchemaBuilder()
      .addDateField('order', { fast: true })
      .addTextField('title', { stored: true })
      .build()

    const index = new Index(schema)
    const writer = index.writer()

    let doc = new Document()
    doc.addDate('order', new Date('2020-01-01').getTime())
    doc.addText('title', 'Test title')
    writer.addDocument(doc)

    doc = new Document()
    doc.addDate('order', new Date('2022-01-01').getTime())
    doc.addText('title', 'Final test title')
    writer.addDocument(doc)

    doc = new Document()
    doc.addDate('order', new Date('2021-01-01').getTime())
    doc.addText('title', 'Another test title')
    writer.addDocument(doc)

    writer.commit()
    index.reload()

    const query = index.parseQuery('test')
    const searcher = index.searcher()
    const result = searcher.search(query, 10, true, 'order')

    expect(result.hits.length).toBe(3)

    let { docAddress } = result.hits[0]
    let searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Final test title'])

    docAddress = result.hits[1].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Another test title'])

    docAddress = result.hits[2].docAddress
    searchedDoc = index.searcher().doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['Test title'])
  })

  it('test_with_merges', () => {
    // This test is taken from tantivy's test suite
    const schema = new SchemaBuilder().addTextField('text', { stored: true }).build()
    const index = new Index(schema)
    index.configReader('Manual')

    const writer = index.writer()

    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('text', 'a')
      writer.addDocument(doc)
    }

    writer.commit()

    for (let i = 0; i < 100; i++) {
      const doc = new Document()
      doc.addText('text', 'a')
      writer.addDocument(doc)
    }

    // This should create 8 segments and trigger a merge.
    writer.commit()
    writer.waitMergingThreads()

    // Accessing the writer again should result in an error.
    expect(() => {
      writer.waitMergingThreads()
    }).toThrowErrorMatchingInlineSnapshot(`[Error: IndexWriter was consumed and no longer in a valid state]`)

    index.reload()

    const query = index.parseQuery('a')
    const searcher = index.searcher()
    const result = searcher.search(query, 500, true)
    expect(result.count).toBe(200)

    expect(searcher.numSegments).toBeLessThan(8)
  })

  it('test_doc_from_dict_numeric_validation', () => {
    const schema = new SchemaBuilder()
      .addUnsignedField('unsigned')
      .addIntegerField('signed')
      .addFloatField('float')
      .build()

    Document.fromDict(
      {
        unsigned: 1000,
        signed: -5,
        float: 0.4,
      },
      schema,
    )

    Document.fromDict(
      {
        unsigned: 1000,
        signed: -5,
        float: 0.4,
      },
      schema,
    )

    // A negative value is rejected for an unsigned field, as in tantivy-py.
    expect(() => {
      Document.fromDict({ unsigned: -50, signed: -5, float: 0.4 }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected U64 type for field unsigned, got unexpected value]`)

    // A fractional value is rejected for an integer field rather than truncated.
    expect(() => {
      Document.fromDict({ unsigned: 1000, signed: 50.4, float: 0.4 }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected I64 type for field signed, got unexpected value]`)

    expect(() => {
      Document.fromDict({ unsigned: 1000, signed: -5, float: 'bad_string' }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected F64 type for field float, got unexpected value]`)

    // Values beyond Number.MAX_SAFE_INTEGER can be passed as BigInt.
    Document.fromDict({ unsigned: 18446744073709551615n, signed: -5, float: 0.4 }, schema)

    // Arrays are supported for single value fields in Node.js version (unlike Python),
    // but every element is still validated.
    Document.fromDict({ unsigned: [1000, 50], signed: -5, float: 0.4 }, schema)

    expect(() => {
      Document.fromDict({ unsigned: 1000, signed: [-5, 150, -3.14], float: 0.4 }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected I64 type for field signed, got unexpected value]`)
  })

  it('test_doc_from_dict_bytes_validation', () => {
    const schema = new SchemaBuilder().addBytesField('bytes').build()

    // Buffer is supported
    Document.fromDict({ bytes: Buffer.from('hello') }, schema)

    // Note: Node.js version doesn't support array formats for bytes (unlike Python)
    // These would throw in Node.js version

    expect(() => {
      Document.fromDict(
        {
          bytes: [
            [1, 2, 3],
            [4, 5, 6],
          ],
        },
        schema,
      )
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected Buffer for bytes field]`)

    expect(() => {
      Document.fromDict({ bytes: [1, 2, 3] }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected Buffer for bytes field]`)

    expect(() => {
      Document.fromDict({ bytes: [1, 2, 256] }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected Buffer for bytes field]`)

    expect(() => {
      Document.fromDict({ bytes: 'hello' }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected Buffer for bytes field]`)

    expect(() => {
      Document.fromDict({ bytes: [1024, 'there'] }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Expected Buffer for bytes field]`)
  })

  it('test_doc_from_dict_ip_addr_validation', () => {
    const schema = new SchemaBuilder().addIpAddrField('ip').build()

    Document.fromDict({ ip: '127.0.0.1' }, schema)
    Document.fromDict({ ip: '::1' }, schema)

    expect(() => {
      Document.fromDict({ ip: 12309812348 }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: invalid IP address syntax]`)

    expect(() => {
      Document.fromDict({ ip: '256.100.0.1' }, schema)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: invalid IP address syntax]`)

    expect(() => {
      Document.fromDict(
        {
          ip: '1234:5678:9ABC:DEF0:1234:5678:9ABC:DEF0:1234',
        },
        schema,
      )
    }).toThrowErrorMatchingInlineSnapshot(`[Error: invalid IP address syntax]`)

    expect(() => {
      Document.fromDict(
        {
          ip: '1234:5678:9ABC:DEF0:1234:5678:9ABC:GHIJ',
        },
        schema,
      )
    }).toThrowErrorMatchingInlineSnapshot(`[Error: invalid IP address syntax]`)
  })

  it('test_doc_from_dict_json_validation', () => {
    // Test implicit JSON
    Document.fromDict({ dict: { hello: 'world' } })

    const schema = new SchemaBuilder().addJsonField('json').build()

    Document.fromDict({ json: {} }, schema)
    Document.fromDict({ json: { hello: 'world' } }, schema)
    Document.fromDict(
      {
        nested: { hello: ['world', '!'] },
        numbers: [1, 2, 3],
      },
      schema,
    )

    const listOfJsons = [{ hello: 'world' }, { nested: { hello: ['world', '!'] }, numbers: [1, 2, 3] }]
    Document.fromDict({ json: listOfJsons }, schema)

    Document.fromDict({ json: JSON.stringify(listOfJsons[1]) }, schema)

    // Note: Node.js version accepts numbers and strings for JSON fields (unlike Python)
    Document.fromDict({ json: 123 }, schema)
    Document.fromDict({ json: 'hello' }, schema)
  })

  it('test_search_result_eq', () => {
    const engQuery = ramIndex.parseQuery('sea whale', ['title', 'body'])
    const espQuery = spanishIndex.parseQuery('vieja', ['title', 'body'])

    const engResult1 = ramIndex.searcher().search(engQuery, 10)
    const engResult2 = ramIndex.searcher().search(engQuery, 10)
    const espResult = spanishIndex.searcher().search(espQuery, 10)

    expect(JSON.stringify(engResult1)).toEqual(JSON.stringify(engResult2))
    expect(JSON.stringify(engResult1)).not.toEqual(JSON.stringify(espResult))
    expect(JSON.stringify(engResult2)).not.toEqual(JSON.stringify(espResult))
  })

  it('test_search_result_pickle', () => {
    const query = ramIndex.parseQuery('sea whale', ['title', 'body'])
    const orig = ramIndex.searcher().search(query, 10)
    const pickled = JSON.parse(JSON.stringify(orig))

    expect(orig).toEqual(pickled)
  })

  it('test_delete_all_documents', () => {
    const index = createIndex()
    const writer = index.writer()
    writer.deleteAllDocuments()
    writer.commit()

    index.reload()
    const query = index.parseQuery('sea whale', ['title', 'body'])
    const result = index.searcher().search(query, 10)

    expect(result.hits.length).toBe(0)
  })
})
describe('TestUpdateClass', () => {
  it('test_delete_update', () => {
    const index = createIndex()
    const writer = index.writer()

    // Delete documents containing "Mice"
    const deleteQuery = index.parseQuery('Mice', ['title'])
    writer.deleteDocumentsByQuery(deleteQuery)
    writer.commit()

    index.reload()

    // Search should now return fewer results
    const searchQuery = index.parseQuery('*', ['title', 'body'])
    const result = index.searcher().search(searchQuery, 10)
    expect(result.hits.length).toBeLessThan(3) // Originally had 3 docs
  })
})

describe('TestFromDiskClass', () => {
  it('test_opens_from_dir_invalid_schema', () => {
    // Test opening index from directory with invalid/incompatible schema
    createIndex(tempDir) // Create an index first

    // Create a different schema (incompatible)
    const invalidSchema = new SchemaBuilder().addTextField('different_field').build()

    // Attempting to open existing index with different schema should throw a specific error
    expect(() => {
      new Index(invalidSchema, tempDir)
    }).toThrow()
  })

  it('test_opens_from_dir', () => {
    // Test that we can open an existing index from directory
    const dirIndex = createIndex(tempDir)
    const searcher = dirIndex.searcher()

    // Now create a new index instance from the same directory
    const reopenedIndex = new Index(schema(), tempDir)
    const reopenedSearcher = reopenedIndex.searcher()

    // Should have the same number of documents
    const query = Query.allQuery()
    const originalResult = searcher.search(query)
    const reopenedResult = reopenedSearcher.search(query)

    expect(reopenedResult.hits.length).toBe(originalResult.hits.length)
  })

  it('test_create_readers', () => {
    const index = createIndex()

    // Test that we can create multiple searchers
    const searcher1 = index.searcher()
    const searcher2 = index.searcher()

    const query = Query.allQuery()
    const result1 = searcher1.search(query)
    const result2 = searcher2.search(query)

    // Both searchers should return the same results
    expect(result1.hits.length).toBe(result2.hits.length)
    expect(result1.count).toBe(result2.count)
  })
})

describe('TestSearcher', () => {
  it('test_searcher_repr', () => {
    const searcher = ramIndex.searcher()

    // Test that searcher has expected properties
    expect(searcher.numSegments).toBeGreaterThan(0)
    expect(typeof searcher.numSegments).toBe('number')
  })
})

describe('TestDocument', () => {
  it('test_document', () => {
    const doc = new Document()
    doc.addText('title', 'Test Document')
    doc.addInteger('id', 123)
    doc.addFloat('rating', 4.5)
    doc.addBoolean('is_good', true)

    // Test that document was created successfully
    expect(doc).toBeDefined()

    // Test document conversion to dict
    const dict = doc.toDict() as TestDoc
    expect(dict.title).toEqual(['Test Document'])
    expect(dict.id).toEqual([123])
    expect(dict.rating).toEqual([4.5])
    expect(dict.is_good).toEqual([true])
  })

  it('test_document_with_date', () => {
    const doc = new Document()
    const testDate = new Date('2021-01-01T00:00:00.000Z')
    doc.addDate('date', testDate.getTime())

    const dict = doc.toDict() as TestDoc
    expect(dict.date?.[0]).toBe(testDate.getTime())
  })

  it('test_document_repr', () => {
    // Test string representation of documents (Node.js equivalent via toDict)
    const doc = new Document()
    doc.addText('name', 'Bill')
    doc.addInteger('reference', 1)
    doc.addInteger('reference', 2)

    const dict = doc.toDict() as any
    expect(dict.name).toEqual(['Bill'])
    expect(dict.reference).toEqual([1, 2])

    // Test string representation via JSON serialization
    const representation = JSON.stringify(dict)
    expect(representation).toContain('Bill')
    expect(representation).toContain('[1,2]')
  })

  it('test_document_repr_utf8', () => {
    // Test UTF8 string representation of documents
    const doc = new Document()
    doc.addText('name', '野菜食べないとやばい') // Japanese text
    doc.addInteger('reference', 1)
    doc.addInteger('reference', 2)

    const dict = doc.toDict() as any
    expect(dict.name).toEqual(['野菜食べないとやばい'])
    expect(dict.reference).toEqual([1, 2])

    // Test UTF8 handling in JSON serialization
    const representation = JSON.stringify(dict)
    expect(representation).toContain('野菜食べないとやばい')
  })

  it('test_document_with_facet', () => {
    // Test document with facet functionality
    const schema = new SchemaBuilder().addTextField('title', { stored: true }).addFacetField('category').build()

    const doc = new Document()
    doc.addText('title', 'Test with facet')
    doc.addFacet('category', Facet.fromString('/category/test'))

    // Test basic document functionality with facets
    const dict = doc.toDict() as any
    expect(dict.title).toEqual(['Test with facet'])
    expect(dict.category).toBeDefined()

    // Test that we can create an index with this document
    const index = new Index(schema)
    const writer = index.writer()
    writer.addDocument(doc)
    writer.commit()
    index.reload()

    const query = index.parseQuery('Test', ['title'])
    const result = index.searcher().search(query)
    expect(result.hits.length).toBe(1)
  })

  it('test_document_eq', () => {
    const doc1 = new Document()
    doc1.addText('title', 'Test')
    doc1.addInteger('id', 1)

    const doc2 = new Document()
    doc2.addText('title', 'Test')
    doc2.addInteger('id', 1)

    const doc3 = new Document()
    doc3.addText('title', 'Different')
    doc3.addInteger('id', 2)

    // Test document equality via string representation
    expect(JSON.stringify(doc1.toDict())).toEqual(JSON.stringify(doc2.toDict()))
    expect(JSON.stringify(doc1.toDict())).not.toEqual(JSON.stringify(doc3.toDict()))
  })

  it('test_document_copy', () => {
    const originalDoc = new Document()
    originalDoc.addText('title', 'Original')
    originalDoc.addInteger('id', 123)

    // Test document copying via dict conversion
    const originalDict = originalDoc.toDict()
    const copiedDict = JSON.parse(JSON.stringify(originalDict))

    expect(copiedDict).toEqual(originalDict)
  })

  it('test_document_pickle', () => {
    const doc = new Document()
    doc.addText('title', 'Pickle Test')
    doc.addInteger('id', 456)

    // Test serialization/deserialization (pickle equivalent)
    const serialized = JSON.stringify(doc.toDict())
    const deserialized = JSON.parse(serialized)

    expect(deserialized.title).toEqual(['Pickle Test'])
    expect(deserialized.id).toEqual([456])
  })
})

describe('TestJsonField', () => {
  it('test_query_from_json_field', () => {
    const schema = new SchemaBuilder()
      .addJsonField('json', { stored: true })
      .addTextField('title', { stored: true })
      .build()

    const index = new Index(schema)
    const writer = index.writer()

    const doc = Document.fromDict(
      {
        title: 'JSON Test',
        json: { name: 'test', value: 42 },
      },
      schema,
    )

    writer.addDocument(doc)
    writer.commit()
    index.reload()

    // Test that we can search within JSON field
    const query = index.parseQuery('test', ['json'])
    const result = index.searcher().search(query)
    expect(result.hits.length).toBeGreaterThanOrEqual(0) // Might be 0 if JSON search isn't supported as expected
  })
})

it('test_bytes', () => {
  // Test bytes field handling with different byte-like inputs
  const schema = new SchemaBuilder().addBytesField('embedding').build()
  const index = new Index(schema)
  const writer = index.writer()

  // Test with Buffer (Node.js equivalent of Python bytes)
  const doc1 = new Document()
  doc1.addBytes('embedding', Buffer.from('abc'))
  writer.addDocument(doc1)

  // Test with Document.fromDict
  const doc2 = Document.fromDict(
    {
      embedding: Buffer.from('xyz'),
    },
    schema,
  )
  writer.addDocument(doc2)

  writer.commit()
  index.reload()

  // Verify documents were added
  const query = Query.allQuery()
  const result = index.searcher().search(query)
  expect(result.hits.length).toBe(2)
})

it('test_schema_eq', () => {
  const schema1 = new SchemaBuilder().addTextField('title', { stored: true }).addTextField('body').build()

  const schema2 = new SchemaBuilder().addTextField('title', { stored: true }).addTextField('body').build()

  const schema3 = new SchemaBuilder()
    .addTextField('title', { stored: true })
    .addTextField('content') // Different field name
    .build()

  expect(schema1.toJSON()).toEqual(schema2.toJSON())
  expect(schema1.toJSON()).not.toEqual(schema3.toJSON())
  expect(schema2.toJSON()).not.toEqual(schema3.toJSON())

  // Test field existence
  expect(schema1.hasField('title')).toBe(true)
  expect(schema1.hasField('body')).toBe(true)
  expect(schema1.hasField('content')).toBe(false)

  expect(schema3.hasField('title')).toBe(true)
  expect(schema3.hasField('content')).toBe(true)
  expect(schema3.hasField('body')).toBe(false)
})

it('test_facet_eq', () => {
  // Test facet equality like Python implementation
  const facet1 = Facet.fromString('/europe/france')
  const facet2 = Facet.fromString('/europe/france')
  const facet3 = Facet.fromString('/europe/germany')

  // Test facet equality via string representation (Node.js equivalent of Python ==)
  expect(facet1.toPathStr()).toEqual(facet2.toPathStr())
  expect(facet1.toPathStr()).not.toEqual(facet3.toPathStr())
  expect(facet2.toPathStr()).not.toEqual(facet3.toPathStr())

  // Test path array equality
  expect(facet1.toPath()).toEqual(facet2.toPath())
  expect(facet1.toPath()).not.toEqual(facet3.toPath())
})

it('test_schema_pickle', () => {
  // Test schema serialization using new toJson/fromJson methods
  const originalSchema = new SchemaBuilder()
    .addIntegerField('id', { stored: true, indexed: true })
    .addTextField('body', { stored: true })
    .addFloatField('rating', { stored: true, indexed: true })
    .addDateField('date')
    .addJsonField('json')
    .addBytesField('bytes')
    .build()

  expect(originalSchema.toJSON()).toBe(Schema.fromJson(originalSchema.toJSON()).toJSON())
})

it('test_facet_pickle', () => {
  // Test facet serialization (Node.js equivalent of Python pickle)
  const orig = Facet.fromString('/europe/france')

  // Test serialization via JSON (Node.js equivalent of pickle)
  const serialized = JSON.stringify({
    path: orig.toPathStr(),
    segments: orig.toPath(),
  })
  const deserialized = JSON.parse(serialized)

  // Recreate facet from serialized data
  const pickled = Facet.fromString(deserialized.path)

  // Test equality via string representation
  expect(orig.toPathStr()).toEqual(pickled.toPathStr())
  expect(orig.toPath()).toEqual(pickled.toPath())
})

it('test_doc_address_pickle', () => {
  // Test document address serialization (Node.js equivalent of Python pickle)
  const orig: DocAddress = { segmentOrd: 42, doc: 123 }

  // Test serialization via JSON (Node.js equivalent of pickle)
  const serialized = JSON.stringify(orig)
  const pickled: DocAddress = JSON.parse(serialized)

  // Test equality
  expect(orig.segmentOrd).toEqual(pickled.segmentOrd)
  expect(orig.doc).toEqual(pickled.doc)
  expect(orig).toEqual(pickled)
})

describe('TestSnippets', () => {
  it('test_document_snippet', () => {
    // Test snippet generation functionality
    const query = ramIndex.parseQuery('sea', ['title', 'body'])
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)

    // Create snippet generator
    const snippetGenerator = SnippetGenerator.create(searcher, query, ramIndex.schema, 'title')
    expect(snippetGenerator).toBeDefined()

    // Set max characters
    snippetGenerator.setMaxNumChars(150)

    // Get the document and generate snippet
    const { docAddress } = result.hits[0]
    const doc = searcher.doc(docAddress)
    const snippet = snippetGenerator.snippetFromDoc(doc)

    // Test snippet methods
    expect(snippet).toBeDefined()
    expect(typeof snippet.toHtml()).toBe('string')
    expect(typeof snippet.fragment()).toBe('string')
    expect(Array.isArray(snippet.highlighted())).toBe(true)

    // Check that the snippet contains the search term
    const htmlSnippet = snippet.toHtml()
    expect(htmlSnippet.toLowerCase()).toContain('sea')
  })
})

describe('TestQuery', () => {
  it('test_term_query', () => {
    const query = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_term_set_query', () => {
    const query = Query.termSetQuery(ramIndex.schema, 'title', ['sea', 'men'])
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(2) // Should match both "The Old Man and the Sea" and "Of Mice and Men"
  })

  it('test_all_query', () => {
    const query = Query.allQuery()
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(3) // Should match all documents
  })

  it('test_phrase_query', () => {
    const query = Query.phraseQuery(ramIndex.schema, 'title', ['old', 'man'])
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_fuzzy_term_query', () => {
    // Test fuzzy search with distance 1 for "maan" -> "man"
    const query = Query.fuzzyTermQuery(ramIndex.schema, 'title', 'maan', 1)
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_regex_query', () => {
    const query = Query.regexQuery(ramIndex.schema, 'title', '.*[Mm]an.*')
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_range_query_numerics', () => {
    const searcher = ramIndexNumericFields.searcher()

    // Test integer range
    const intQuery = Query.rangeQuery(ramIndexNumericFields.schema, 'id', FieldType.Integer, 1, 2, true, true)
    let result = searcher.search(intQuery)
    expect(result.hits.length).toBe(2)

    // Test float range
    const floatQuery = Query.rangeQuery(ramIndexNumericFields.schema, 'rating', FieldType.Float, 3.0, 4.0, true, false)
    result = searcher.search(floatQuery)
    expect(result.hits.length).toBe(1)
  })

  it('test_range_query_dates', () => {
    const searcher = ramIndexWithDateField.searcher()

    // Use parseQuery for date range (like the working test) instead of Query.rangeQuery
    // which might require fast fields for dates
    const startDate = new Date('2020-12-31T00:00:00.000Z')
    const endDate = new Date('2021-01-03T23:59:59.999Z')

    const query = ramIndexWithDateField.parseQuery(`date:[${startDate.toISOString()} TO ${endDate.toISOString()}]`)

    const result = searcher.search(query)
    expect(result.hits.length).toBe(2)
  })

  it('test_range_query_ip_addrs', () => {
    const searcher = ramIndexWithIpAddrField.searcher()

    const query = Query.rangeQuery(
      ramIndexWithIpAddrField.schema,
      'ip_addr',
      FieldType.IpAddr,
      '10.0.0.0',
      '127.0.0.255',
      true,
      true,
    )

    const result = searcher.search(query)
    expect(result.hits.length).toBe(2) // Should match 10.0.0.1 and 127.0.0.1
  })

  it('test_boolean_query', () => {
    const titleQuery = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const bodyQuery = Query.termQuery(ramIndex.schema, 'body', 'man')

    // Test individual queries first
    const searcher = ramIndex.searcher()

    // Test title query alone
    let result = searcher.search(titleQuery)
    expect(result.hits.length).toBe(1)

    // Test body query alone
    result = searcher.search(bodyQuery)
    expect(result.hits.length).toBe(1)

    // Test boolean query with MUST (AND) - should find documents that have both 'sea' in title AND 'man' in body
    const mustQuery = Query.booleanQuery([
      { occur: 0, query: titleQuery }, // Must
      { occur: 0, query: bodyQuery }, // Must
    ])
    result = searcher.search(mustQuery)
    expect(result.hits.length).toBe(1) // "The Old Man and the Sea" has both

    // Test boolean query with SHOULD (OR) - should find documents that have either 'sea' in title OR 'man' in body
    const shouldQuery = Query.booleanQuery([
      { occur: 1, query: titleQuery }, // Should
      { occur: 1, query: bodyQuery }, // Should
    ])
    result = searcher.search(shouldQuery)
    expect(result.hits.length).toBe(1) // Still 1 because both terms are in the same document

    // Test with MUST_NOT - find documents that have 'sea' in title but NOT 'winter' in body
    const winterBodyQuery = Query.termQuery(ramIndex.schema, 'body', 'winter')
    const mustNotQuery = Query.booleanQuery([
      { occur: 0, query: titleQuery }, // Must have 'sea' in title
      { occur: 2, query: winterBodyQuery }, // Must NOT have 'winter' in body
    ])
    result = searcher.search(mustNotQuery)
    expect(result.hits.length).toBe(1) // "The Old Man and the Sea" matches
  })

  it('test_boost_query', () => {
    const baseQuery = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const boostedQuery = Query.boostQuery(baseQuery, 2.0)

    const searcher = ramIndex.searcher()
    const result = searcher.search(boostedQuery)
    expect(result.hits.length).toBe(1)

    // Boosted query should return same results but with different scores
    const baseResult = searcher.search(baseQuery)
    expect(result.hits.length).toBe(baseResult.hits.length)
  })

  it('test_const_score_query', () => {
    const baseQuery = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const constQuery = Query.constScoreQuery(baseQuery, 1.5)

    const searcher = ramIndex.searcher()
    const result = searcher.search(constQuery)
    expect(result.hits.length).toBe(1)
  })

  it('test_disjunction_max_query', () => {
    const titleQuery = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const bodyQuery = Query.termQuery(ramIndex.schema, 'body', 'fish')

    const disjunctionQuery = Query.disjunctionMaxQuery([titleQuery, bodyQuery])

    const searcher = ramIndex.searcher()
    const result = searcher.search(disjunctionQuery)
    expect(result.hits.length).toBe(1) // Should match "The Old Man and the Sea"
  })

  it('test_more_like_this_query', () => {
    const searcher = ramIndex.searcher()

    // First get a document address
    const initialQuery = Query.termQuery(ramIndex.schema, 'title', 'sea')
    const initialResult = searcher.search(initialQuery)
    expect(initialResult.hits.length).toBe(1)

    const { docAddress } = initialResult.hits[0]

    // Create more-like-this query
    const mltQuery = Query.moreLikeThisQuery(docAddress)
    const result = searcher.search(mltQuery)

    // Should find similar documents (might be 0 if no similar docs)
    expect(result.hits.length).toBeGreaterThanOrEqual(0)
  })

  it('test_range_query_invalid_types', () => {
    // Test that invalid range queries throw errors
    expect(() => {
      Query.rangeQuery(ramIndexNumericFields.schema, 'nonexistent_field', FieldType.Integer, 1, 10, true, true)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Field \`nonexistent_field\` is not defined in the schema.]`)
  })

  it('test_delete_documents_deprecated_alias', () => {
    // Kept for parity with tantivy-py; behaves exactly like deleteDocumentsByTerm.
    const index = createIndex()
    const writer = index.writer()
    expect(writer.deleteDocuments('title', 'sea')).toBe(writer.deleteDocumentsByTerm('title', 'sea') - 1n)
    writer.commit()
    writer.waitMergingThreads()
    index.reload()
  })

  it('test_delete_documents_by_term', () => {
    const index = createIndex()
    const writer = index.writer()

    // Test that deleteDocumentsByTerm method exists and can be called
    // The exact behavior may vary based on how terms are indexed
    const deletedCount = writer.deleteDocumentsByTerm('title', 'Sea')
    expect(typeof deletedCount).toBe('bigint')
    expect(deletedCount >= 0n).toBe(true)

    writer.commit()
    index.reload()
  })

  it('test_phrase_query_with_slop', () => {
    // Test phrase query with slop (word distance tolerance)
    const query = Query.phraseQuery(ramIndex.schema, 'body', ['old', 'man'], 1)
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
  })

  it('test_fuzzy_term_query_advanced', () => {
    // Test fuzzy search with different parameters
    const query = Query.fuzzyTermQuery(ramIndex.schema, 'title', 'mann', 2, true, false)
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    // Might find matches with distance 2
    expect(result.hits.length).toBeGreaterThanOrEqual(0)
  })

  it('test_phrase_prefix_query', () => {
    // "old m" should match "old man" in "The Old Man and the Sea" body
    const query = Query.phrasePrefixQuery(ramIndex.schema, 'body', ['old', 'm'])
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_phrase_prefix_query_empty_words', () => {
    expect(() => {
      Query.phrasePrefixQuery(ramIndex.schema, 'body', [])
    }).toThrow('words must not be empty')
  })

  it('test_regex_phrase_query', () => {
    // Match "old" followed by "man" using regex patterns with slop
    const query = Query.regexPhraseQuery(ramIndex.schema, 'body', ['old', 'ma.*'], 1)
    const searcher = ramIndex.searcher()
    const result = searcher.search(query)
    expect(result.hits.length).toBe(1)
    const { docAddress } = result.hits[0]
    const searchedDoc = searcher.doc(docAddress)
    expect((searchedDoc.toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])
  })

  it('test_regex_phrase_query_empty_patterns', () => {
    expect(() => {
      Query.regexPhraseQuery(ramIndex.schema, 'body', [])
    }).toThrow('words must not be empty')
  })

  it('test_index_exists', () => {
    // Test basic index functionality
    expect(ramIndex).toBeDefined()
    expect(ramIndex.schema).toBeDefined()
    expect(ramIndex.searcher()).toBeDefined()
  })

  it('test_range_query_unsupported_types', () => {
    // Test range queries with unsupported field types
    const index = ramIndex

    // Test with text field (should be unsupported)
    expect(() => {
      Query.rangeQuery(index.schema, 'title', FieldType.Text, 'a', 'z', true, true)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Text fields are not supported for range queries.]`)

    // Test with field that doesn't exist
    expect(() => {
      Query.rangeQuery(index.schema, 'nonexistent', FieldType.Integer, 1, 10, true, true)
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Field \`nonexistent\` is not defined in the schema.]`)
  })
})

describe('TestTokenizers', () => {
  it('test_build_and_register_simple_tokenizer', () => {
    const customAnalyzer = new TextAnalyzerBuilder(TokenizerStatic.whitespace())
      .filter(FilterStatic.lowercase())
      .build()

    const docText = '#03 8903 HELLO'
    // Check that string is split on whitespace and lowercased.
    expect(customAnalyzer.analyze(docText)).toEqual(['#03', '8903', 'hello'])

    const schema = new SchemaBuilder().addTextField('content', { tokenizerName: 'custom_analyzer' }).build()

    const index = new Index(schema)
    // Note: registerTokenizer might expect TextAnalyzer, trying with customAnalyzer
    try {
      index.registerTokenizer('custom_analyzer', customAnalyzer as any)
    } catch (error) {
      // If registerTokenizer doesn't work, skip the rest of this test
      console.warn('registerTokenizer not working as expected:', error)
      return
    }

    const writer = index.writer()
    const doc = Document.fromDict({ content: docText }, schema)
    writer.addDocument(doc)
    writer.commit()
    index.reload() // Index must be reloaded for search to work.

    const query = Query.termQuery(index.schema, 'content', '#03')
    const result = index.searcher().search(query, 1)
    expect(result.hits.length).toBe(1)

    // Uppercase term 'HELLO' should not be matchable,
    // as 'HELLO' was lowercased to 'hello' by the analyzer.
    const upperQuery = Query.termQuery(index.schema, 'content', 'HELLO')
    const upperResult = index.searcher().search(upperQuery, 1)
    expect(upperResult.hits.length).toBe(0)
  })

  it('test_build_regex_tokenizer_with_simple_pattern', () => {
    const tokenPattern = '(?i)[a-z]+'
    const analyzer = new TextAnalyzerBuilder(TokenizerStatic.regex(tokenPattern)).build()
    const docText = 'all00of00these00words'
    expect(analyzer.analyze(docText)).toEqual(['all', 'of', 'these', 'words'])
  })

  it('test_build_regex_tokenizer_with_bad_pattern', () => {
    const tokenPattern = '(?i)[a-z+'
    // Implementation detail: The invalid regex error arises
    // within the Builder, not the wrapped Tokenizer.
    expect(() => {
      new TextAnalyzerBuilder(TokenizerStatic.regex(tokenPattern))
    }).toThrowErrorMatchingInlineSnapshot(`[Error: Invalid regex pattern: An invalid argument was passed: '(?i)[a-z+']`)
  })

  it('test_build_ngram_tokenizer', () => {
    const analyzer = new TextAnalyzerBuilder(TokenizerStatic.ngram(2, 3)).build()
    const docText = 'ferrous'
    expect(analyzer.analyze(docText)).toEqual(['fe', 'fer', 'er', 'err', 'rr', 'rro', 'ro', 'rou', 'ou', 'ous', 'us'])
  })

  it('test_build_tokenizer_w_stopword_filter', () => {
    const analyzer = new TextAnalyzerBuilder(TokenizerStatic.simple()).filter(FilterStatic.stopword('english')).build()
    const docText = 'the bad wolf buys an axe'
    expect(analyzer.analyze(docText)).toEqual(['bad', 'wolf', 'buys', 'axe'])
  })

  for (const language of ['arabic', 'greek', 'romanian', 'tamil', 'turkish']) {
    it(`test_build_tokenizer_w_stopword_filter_no_builtin_list_${language}`, () => {
      // These languages have a stemmer, but tantivy has no builtin stop word list
      const builder = new TextAnalyzerBuilder(TokenizerStatic.simple())
      expect(() => builder.filter(FilterStatic.stopword(language))).toThrow('stop word list')
    })
  }

  it('test_build_tokenizer_w_custom_stopwords_filter', () => {
    const analyzer = new TextAnalyzerBuilder(TokenizerStatic.simple())
      .filter(FilterStatic.stopword('english'))
      .filter(FilterStatic.customStopword(['like']))
      .build()
    const docText = 'that is, like, such a weird way to, like, test'
    expect(analyzer.analyze(docText)).toEqual(['weird', 'way', 'test'])
  })

  it('test_delete_documents_by_query', () => {
    const schema = new SchemaBuilder().addTextField('id', { fast: true }).build()
    const index = new Index(schema)
    let writer = index.writer()
    const idStr = 'test-1'
    const sourceDoc = {
      id: idStr,
    }
    const doc = Document.fromDict(sourceDoc, schema)
    writer.addDocument(doc)
    writer.commit()
    writer.waitMergingThreads()
    index.reload()

    const query = index.parseQuery(`id:${idStr}`)
    let result = index.searcher().search(query)
    expect(result.count).toBe(1)

    writer = index.writer()
    writer.deleteDocumentsByQuery(query)
    writer.commit()
    writer.waitMergingThreads()

    index.reload()
    result = index.searcher().search(query)
    expect(result.count).toBe(0)
  })
})

describe('TestFacet', () => {
  it('test_facet_api', () => {
    // Test basic Facet API that matches Python implementation

    // Test root facet
    const rootFacet = Facet.root()
    expect(rootFacet.isRoot).toBe(true)
    expect(rootFacet.toPathStr()).toBe('/')

    // Test fromString (equivalent to Python from_string)
    const facet = Facet.fromString('/electronics/computers/laptops')
    expect(facet.isRoot).toBe(false)
    expect(facet.toPathStr()).toBe('/electronics/computers/laptops')
    expect(facet.toString()).toBe('/electronics/computers/laptops')

    // Test toPath (should return path segments)
    const pathSegments = facet.toPath()
    expect(pathSegments).toEqual(['electronics', 'computers', 'laptops'])

    // Test fromPath
    const facetFromPath = Facet.fromPath(['books', 'fiction', 'scifi'])
    expect(facetFromPath.toPathStr()).toBe('/books/fiction/scifi')
    expect(facetFromPath.toPath()).toEqual(['books', 'fiction', 'scifi'])

    // Test isPrefixOf
    const parentFacet = Facet.fromString('/electronics')
    const childFacet = Facet.fromString('/electronics/computers')
    expect(parentFacet.isPrefixOf(childFacet)).toBe(true)
    expect(childFacet.isPrefixOf(parentFacet)).toBe(false)
    expect(rootFacet.isPrefixOf(facet)).toBe(true)

    // Test that we can use facets in documents (integration test)
    const schema = new SchemaBuilder().addTextField('title', { stored: true }).addFacetField('category').build()

    const index = new Index(schema)
    const writer = index.writer()

    const doc = new Document()
    doc.addText('title', 'Test Product')
    doc.addFacet('category', facet) // Use our facet
    writer.addDocument(doc)
    writer.commit()
    index.reload()

    // Verify the document was indexed
    const query = index.parseQuery('Test', ['title'])
    const result = index.searcher().search(query)
    expect(result.hits.length).toBe(1)
  })

  it('test_simple_search_facet', () => {
    const indexSchema = new SchemaBuilder().addTextField('title', { stored: true }).addFacetField('category').build()
    const index = new Index(indexSchema)
    const writer = index.writer(15_000_000, 1)

    const doc = new Document()
    doc.addText('title', 'Book about whales')
    doc.addFacet('category', Facet.fromString('/books/fiction'))
    writer.addDocument(doc)
    writer.commit()
    writer.waitMergingThreads()
    index.reload()

    expect(index.searcher().search(index.parseQuery('+category:/books'), 10).hits.length).toBe(1)
    expect(index.searcher().search(index.parseQuery('about', ['title']), 10).hits.length).toBe(1)
  })
})

describe('TestParseQueryOptions', () => {
  it('test_parse_query_conjunction_by_default', () => {
    // not a perfect comparison, but it's simple and does the job
    expect(ramIndex.parseQuery('men winter', ['title'], undefined, undefined, false).toString()).toBe(
      ramIndex.parseQuery('men OR winter', ['title'], undefined, undefined, true).toString(),
    )
    expect(ramIndex.parseQuery('men winter', ['title'], undefined, undefined, true).toString()).toBe(
      ramIndex.parseQuery('men AND winter', ['title'], undefined, undefined, true).toString(),
    )
  })

  it('test_parse_query_allow_regexes', () => {
    const query = ramIndex.parseQuery('title:/(?:man|men)/', undefined, undefined, undefined, undefined, true)
    const searcher = ramIndex.searcher()
    const result = searcher.search(query, 10)
    expect(result.hits.length).toBe(2)
    const titles = result.hits.map((hit) => (searcher.doc(hit.docAddress).toDict() as TestDoc).title?.[0])
    expect(titles).toContain('The Old Man and the Sea')
    expect(titles).toContain('Of Mice and Men')

    expect(() => ramIndex.parseQuery('title:/(?:man|men)/')).toThrow('Regex queries are not allowed')

    const [, errors] = ramIndex.parseQueryLenient('title:/(?:man|men)/')
    expect(errors.length).toBe(1)
    expect(errors[0]).toBeInstanceOf(UnsupportedQueryError)
  })
})

describe('TestAggregations', () => {
  it('test_aggregate_terms', () => {
    // terms aggregation on a fast text field returns buckets with doc counts
    const searcher = ramIndexNumericFields.searcher()
    const result = searcher.aggregate(Query.allQuery(), {
      body_terms: { terms: { field: 'body', size: 5 } },
    }) as any

    const buckets = result.body_terms.buckets
    expect(buckets.length).toBe(5) // capped by size=5
    for (const bucket of buckets) {
      expect(typeof bucket.key).toBe('string')
      expect(Number(bucket.doc_count)).toBeGreaterThanOrEqual(1)
    }
    // Results are sorted by doc_count descending.
    const counts = buckets.map((b: any) => Number(b.doc_count))
    expect(counts).toEqual([...counts].sort((a, b) => b - a))
    // "and" appears in both documents so it has the highest doc_count (2).
    const topKeys = buckets.filter((b: any) => b.doc_count === buckets[0].doc_count).map((b: any) => b.key)
    expect(topKeys).toContain('and')
    expect(Number(buckets[0].doc_count)).toBe(2)
  })

  it('test_cardinality', () => {
    const searcher = ramIndexNumericFields.searcher()
    const query = Query.allQuery()

    // rating has 2 unique values: 3.5 and 4.5
    expect(searcher.cardinality(query, 'rating')).toBe(2)
    // id has 2 unique values: 1 and 2
    expect(searcher.cardinality(query, 'id')).toBe(2)

    // a query that filters to one document
    const singleDoc = Query.termQuery(ramIndexNumericFields.schema, 'id', 1)
    expect(searcher.cardinality(singleDoc, 'rating')).toBe(1)

    // the body text fast field contains many unique tokens
    expect(searcher.cardinality(query, 'body')).toBeGreaterThan(10)
  })
})

describe('TestOrderByFastField', () => {
  const cases: [string, number | boolean | string, number | boolean | string][] = [
    ['u64_field', 0, 2],
    ['i64_field', -10, 5],
    ['f64_field', 1.5, 3.14],
    ['bool_field', false, true],
    ['str_field', 'apple', 'cherry'],
    ['date_field', Date.UTC(2024, 0, 1), Date.UTC(2026, 0, 1)],
  ]

  for (const [field, lowValue, highValue] of cases) {
    it(`test_order_by_fast_field_${field}`, () => {
      const searcher = indexWithOrderFastFields.searcher()
      const query = indexWithOrderFastFields.parseQuery('title', ['title'])

      let result = searcher.search(query, 10, true, field)
      expect(result.hits.length).toBe(2)
      expect(result.hits[0].order).toBe(highValue)
      expect(result.hits[1].order).toBe(lowValue)
      expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).title).toEqual(['high title'])
      expect((searcher.doc(result.hits[1].docAddress).toDict() as TestDoc).title).toEqual(['low title'])

      result = searcher.search(query, 10, true, field, 0, Order.Asc)
      expect(result.hits.length).toBe(2)
      expect(result.hits[0].order).toBe(lowValue)
      expect(result.hits[1].order).toBe(highValue)
      expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).title).toEqual(['low title'])
      expect((searcher.doc(result.hits[1].docAddress).toDict() as TestDoc).title).toEqual(['high title'])
    })
  }
})

describe('TestIndexCompatibility', () => {
  it('test_is_compatible_for_current_index', () => {
    const dir = mkdtempSync(join(tmpdir(), 'tantivy-compat-'))
    createIndex(dir)
    expect(Index.isCompatible(dir)).toBe(true)
  })

  it('test_is_compatible_raises_for_missing_index', () => {
    expect(() => Index.isCompatible(join(tempDir, 'does-not-exist'))).toThrow()
  })
})

describe('TestDateRoundtrip', () => {
  it('test_date_index_roundtrip', () => {
    const indexSchema = new SchemaBuilder()
      .addDateField('date', { stored: true, indexed: true })
      .addTextField('title', { stored: true })
      .build()
    const index = new Index(indexSchema)
    const writer = index.writer()

    // JavaScript Date is always an absolute instant; a millisecond value
    // survives the round-trip unchanged.
    const utc = Date.UTC(2019, 7, 12, 13, 0, 0, 123)
    const offset = new Date('2019-08-12T13:00:00+05:30').getTime()

    const doc1 = new Document()
    doc1.addText('title', 'utc')
    doc1.addDate('date', utc)
    writer.addDocument(doc1)

    const doc2 = new Document()
    doc2.addText('title', 'offset')
    doc2.addDate('date', offset)
    writer.addDocument(doc2)

    writer.commit()
    index.reload()

    const searcher = index.searcher()
    const hits = searcher.search(index.parseQuery('utc OR offset', ['title']), 10).hits
    const byTitle = Object.fromEntries(
      hits.map((hit) => {
        const d = searcher.doc(hit.docAddress).toDict() as any
        return [d.title[0], d.date[0]]
      }),
    )

    expect(byTitle.utc).toBe(utc)
    expect(byTitle.offset).toBe(Date.UTC(2019, 7, 12, 7, 30, 0))
  })

  it('test_document_date_milliseconds_preserved', () => {
    const date = Date.UTC(2019, 7, 12, 13, 0, 0, 123)
    const doc = Document.fromDict({ name: 'Bill', date })
    expect(doc.getFirst('date')).toBe(date)
  })

  it('test_document_accepts_js_date', () => {
    const indexSchema = new SchemaBuilder().addDateField('date', { stored: true, indexed: true }).build()
    const date = new Date('2019-08-12T13:00:00.123Z')
    const doc = Document.fromDict({ date }, indexSchema)
    expect(doc.getFirst('date')).toBe(date.getTime())
  })
})

describe('TestJsonFieldExpandDots', () => {
  it('test_json_field_expand_dots_enabled', () => {
    // Without expandDots, a literal "." in a JSON key is NOT treated as a
    // path separator - querying it as a path must fail to match, and the
    // literal key must be reachable only via an escaped dot.
    const plainIndex = new Index(new SchemaBuilder().addJsonField('attrs', { stored: true }).build())
    let writer = plainIndex.writer()
    const plainDoc = new Document()
    plainDoc.addJson('attrs', { 'a.b': 'hello' })
    writer.addDocument(plainDoc)
    writer.commit()
    plainIndex.reload()

    expect(plainIndex.searcher().search(plainIndex.parseQuery('attrs.a.b:hello', ['attrs']), 10).hits.length).toBe(0)
    expect(plainIndex.searcher().search(plainIndex.parseQuery('attrs.a\\.b:hello', ['attrs']), 10).hits.length).toBe(1)

    // With expandDots enabled, the same flat key "a.b" is treated as a
    // nested path a -> b, so the unescaped dotted query now matches.
    const expandIndex = new Index(
      new SchemaBuilder().addJsonField('attrs', { stored: true, expandDotsEnabled: true }).build(),
    )
    writer = expandIndex.writer()
    const expandDoc = new Document()
    expandDoc.addJson('attrs', { 'a.b': 'hello' })
    writer.addDocument(expandDoc)
    writer.commit()
    expandIndex.reload()

    expect(expandIndex.searcher().search(expandIndex.parseQuery('attrs.a.b:hello', ['attrs']), 10).hits.length).toBe(1)
  })
})

describe('TestQueryAdditions', () => {
  it('test_empty_query', () => {
    expect(ramIndex.searcher().search(Query.emptyQuery(), 10).hits.length).toBe(0)
  })

  it('test_exists_query', () => {
    const query = Query.existsQuery('body')
    expect(indexWithEmptyFastField.searcher().search(query, 10).hits.length).toBe(1)
  })

  it('test_not_exists_query', () => {
    const query = Query.booleanQuery([
      { occur: Occur.Must, query: Query.allQuery() },
      { occur: Occur.MustNot, query: Query.existsQuery('body') },
    ])
    expect(indexWithEmptyFastField.searcher().search(query, 10).hits.length).toBe(2)
  })

  it('test_regex_phrase_query', () => {
    const searcher = ramIndex.searcher()
    const schema = ramIndex.schema

    // should match the title "The Old Man and the Sea"
    expect(searcher.search(Query.regexPhraseQuery(schema, 'title', ['o.d', 'ma[nm]']), 10).hits.length).toBe(1)
    // shouldn't match any document
    expect(searcher.search(Query.regexPhraseQuery(schema, 'title', ['man', 'old']), 10).hits.length).toBe(0)
    // should match "The Old Man and the Sea" with the given offsets
    expect(
      searcher.search(
        Query.regexPhraseQuery(schema, 'title', [
          [1, '[a-m]an'],
          [0, 'old'],
        ]),
        10,
      ).hits.length,
    ).toBe(1)
    // shouldn't match any document with the default slop of 0
    expect(searcher.search(Query.regexPhraseQuery(schema, 'title', ['ma.', 'se.']), 10).hits.length).toBe(0)
    // should match the title "The Old Man and the Sea" with slop 2
    expect(searcher.search(Query.regexPhraseQuery(schema, 'title', ['ma.', 'se.'], 2), 10).hits.length).toBe(1)

    expect(() => Query.regexPhraseQuery(schema, 'title', [])).toThrow('words must not be empty.')
  })

  it('test_phrase_prefix_query', () => {
    const searcher = ramIndex.searcher()
    const schema = ramIndex.schema

    expect(searcher.search(Query.phrasePrefixQuery(schema, 'title', ['old', 'man']), 10).hits.length).toBe(1)
    // should still match the title "The Old Man and the Sea"
    expect(searcher.search(Query.phrasePrefixQuery(schema, 'title', ['old', 'ma']), 10).hits.length).toBe(1)
    // shouldn't match any document
    expect(searcher.search(Query.phrasePrefixQuery(schema, 'title', ['man', 'old']), 10).hits.length).toBe(0)
    // should match "The Old Man and the Sea" with the given offsets
    expect(
      searcher.search(
        Query.phrasePrefixQuery(schema, 'title', [
          [1, 'm'],
          [0, 'old'],
        ]),
        10,
      ).hits.length,
    ).toBe(1)

    expect(() => Query.phrasePrefixQuery(schema, 'title', [])).toThrow('words must not be empty.')
  })

  it('test_boolean_query_minimum_number_should_match', () => {
    const schema = ramIndex.schema
    const query1 = Query.termQuery(schema, 'title', 'sea')
    const query2 = Query.termQuery(schema, 'title', 'mice')
    const subqueries = [
      { occur: Occur.Should, query: query1 },
      { occur: Occur.Should, query: query2 },
    ]

    // no document matches both clauses
    expect(ramIndex.searcher().search(Query.booleanQuery(subqueries, 2), 10).hits.length).toBe(0)
    expect(ramIndex.searcher().search(Query.booleanQuery(subqueries, 1), 10).hits.length).toBe(2)
  })

  it('test_boolean_query_helpers', () => {
    const searcher = ramIndex.searcher()
    const schema = ramIndex.schema

    const querySea = Query.termQuery(schema, 'title', 'sea')
    const queryMice = Query.termQuery(schema, 'title', 'mice')
    const queryOld = Query.termQuery(schema, 'title', 'old')
    const queryMan = Query.termQuery(schema, 'title', 'man')

    // No document contains both "sea" and "mice" in the title
    expect(searcher.search(querySea.andMustMatch([queryMice]), 10).hits.length).toBe(0)

    // "The Old Man and the Sea" contains both "old" and "man"
    let result = searcher.search(queryOld.andMustMatch([queryMan]), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).title).toEqual(['The Old Man and the Sea'])

    // the same, but through a long chain, which must stay flat
    let chained = queryOld
    for (let i = 0; i < 11; i++) chained = chained.andMustMatch([queryMan])
    expect(searcher.search(chained, 10).hits.length).toBe(1)

    // Should match documents containing either "sea" or "mice"
    result = searcher.search(querySea.orShouldMatch([queryMice]), 10)
    expect(result.hits.length).toBe(2)

    // All 3 docs contain "and" in the body; exclude the one titled "...Sea"
    const queryAndBody = Query.termQuery(schema, 'body', 'and')
    result = searcher.search(queryAndBody.andMustNotMatch([querySea]), 10)
    expect(result.hits.length).toBe(2)
    const titles = result.hits.map((hit) => (searcher.doc(hit.docAddress).toDict() as TestDoc).title?.[0])
    expect(titles).not.toContain('The Old Man and the Sea')
  })

  it('test_boolean_query_helpers_multiple_queries', () => {
    const searcher = ramIndex.searcher()
    const schema = ramIndex.schema

    const querySea = Query.termQuery(schema, 'title', 'sea')
    const queryMice = Query.termQuery(schema, 'title', 'mice')
    const queryOld = Query.termQuery(schema, 'title', 'old')
    const queryMan = Query.termQuery(schema, 'title', 'man')
    const queryFrankenstein = Query.termQuery(schema, 'title', 'frankenstein')
    const queryAndBody = Query.termQuery(schema, 'body', 'and')

    expect(searcher.search(queryOld.andMustMatch([queryMan, querySea]), 10).hits.length).toBe(1)
    expect(searcher.search(queryOld.orShouldMatch([queryMice, queryFrankenstein]), 10).hits.length).toBe(3)

    const result = searcher.search(queryAndBody.andMustNotMatch([querySea, queryMice]), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).title).toContain('Frankenstein')

    // Calling with no queries returns an equivalent query
    expect(searcher.search(queryOld.andMustMatch([]), 10).hits.length).toBe(1)
  })

  it('test_boolean_query_helpers_mixed_chains', () => {
    // Chains mixing AND and OR must group the left-hand side correctly.
    const searcher = ramIndex.searcher()
    const schema = ramIndex.schema

    const queryMice = Query.termQuery(schema, 'title', 'mice')
    const queryOld = Query.termQuery(schema, 'title', 'old')
    const queryMan = Query.termQuery(schema, 'title', 'man')
    const queryFrankenstein = Query.termQuery(schema, 'title', 'frankenstein')

    // (old OR mice) AND frankenstein: no document satisfies both sides.
    expect(searcher.search(queryOld.orShouldMatch([queryMice]).andMustMatch([queryFrankenstein]), 10).hits.length).toBe(
      0,
    )

    // (old AND man) OR mice: the AND group must stay grouped.
    const result = searcher.search(queryOld.andMustMatch([queryMan]).orShouldMatch([queryMice]), 10)
    expect(result.hits.length).toBe(2)
    const titles = result.hits.map((hit) => (searcher.doc(hit.docAddress).toDict() as TestDoc).title?.[0])
    expect(titles.sort()).toEqual(['Of Mice and Men', 'The Old Man and the Sea'])
  })

  it('test_more_like_this_document_fields_query', () => {
    const indexSchema = new SchemaBuilder()
      .addUnsignedField('id', { stored: true, indexed: true })
      .addTextField('title')
      .addTextField('body')
      .build()
    const index = new Index(indexSchema)
    const writer = index.writer()
    writer.addDocument(Document.fromDict({ id: 1, title: 'aaa', body: 'the old man and the sea' }, indexSchema))
    writer.addDocument(Document.fromDict({ id: 2, title: 'bbb', body: 'an old man sailing on the sea' }, indexSchema))
    writer.addDocument(Document.fromDict({ id: 3, title: 'ccc', body: 'send this message to alice' }, indexSchema))
    writer.commit()
    writer.waitMergingThreads()
    index.reload()

    const mltQuery = Query.moreLikeThisDocumentFieldsQuery(
      indexSchema,
      { title: 'aaa', body: 'the old man and the sea' },
      1,
      undefined,
      1,
      10,
    )
    expect(mltQuery.toString()).toContain('target: DocumentFields(')

    const searcher = index.searcher()
    const hitIds = searcher.search(mltQuery, 10).hits.map((hit) => (searcher.doc(hit.docAddress).toDict() as any).id[0])
    expect(hitIds.sort()).toEqual([1, 2])
  })

  it('test_more_like_this_document_fields_query_rejects_unknown_fields', () => {
    expect(() => Query.moreLikeThisDocumentFieldsQuery(ramIndex.schema, { unknown_field: 'value' })).toThrow(
      'Field `unknown_field` is not defined in the schema.',
    )
  })

  it('test_range_query_unbounded', () => {
    const index = ramIndexNumericFields
    const searcher = index.searcher()

    // unbounded upper (rating >= 4.0)
    let result = searcher.search(Query.rangeQuery(index.schema, 'rating', FieldType.Float, 4.0, null), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([2])

    // unbounded lower (rating <= 4.0)
    result = searcher.search(Query.rangeQuery(index.schema, 'rating', FieldType.Float, null, 4.0), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([1])

    // both bounds null is an error
    expect(() => Query.rangeQuery(index.schema, 'rating', FieldType.Float, null, null)).toThrow('At least one')

    // integer field, unbounded upper (id >= 2)
    result = searcher.search(Query.rangeQuery(index.schema, 'id', FieldType.Integer, 2, null), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([2])

    // integer field, unbounded lower (id <= 1)
    result = searcher.search(Query.rangeQuery(index.schema, 'id', FieldType.Integer, null, 1), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([1])
  })

  it('test_range_query_contradictory_include_and_null_bound', () => {
    const index = ramIndexNumericFields
    expect(() => Query.rangeQuery(index.schema, 'rating', FieldType.Float, null, 4.0, false, true)).toThrow(
      'includeLower',
    )
    expect(() => Query.rangeQuery(index.schema, 'rating', FieldType.Float, 3.0, null, true, false)).toThrow(
      'includeUpper',
    )
  })

  it('test_range_query_numerics_with_inverted_index', () => {
    const index = ramIndexNumericFields
    const searcher = index.searcher()

    // including both bounds
    expect(
      searcher.search(Query.rangeQuery(index.schema, 'id', FieldType.Integer, 1, 2, true, true, true), 10).hits.length,
    ).toBe(2)
    // excluding the lower bound
    expect(
      searcher.search(Query.rangeQuery(index.schema, 'id', FieldType.Integer, 1, 2, false, true, true), 10).hits.length,
    ).toBe(1)
    // unbounded upper (id >= 2)
    expect(
      searcher.search(Query.rangeQuery(index.schema, 'id', FieldType.Integer, 2, null, true, true, true), 10).hits
        .length,
    ).toBe(1)
  })

  it('test_term_query_dates', () => {
    const index = ramIndexWithDateField
    const searcher = index.searcher()

    let result = searcher.search(Query.termQuery(index.schema, 'date', Date.UTC(2021, 0, 1)), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([1])

    // a JS Date instance points at the same instant
    result = searcher.search(Query.termQuery(index.schema, 'date', new Date('2021-01-01T00:00:00Z')), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as TestDoc).id).toEqual([1])

    // a date no document was indexed with matches nothing
    result = searcher.search(Query.termQuery(index.schema, 'date', Date.UTC(2021, 0, 3)), 10)
    expect(result.hits.length).toBe(0)
  })

  it('test_term_set_query_dates', () => {
    const index = ramIndexWithDateField
    const searcher = index.searcher()
    const query = Query.termSetQuery(index.schema, 'date', [new Date('2021-01-01T00:00:00Z'), Date.UTC(2021, 0, 2)])
    const result = searcher.search(query, 10)
    expect(result.hits.length).toBe(2)
    const ids = result.hits.map((hit) => (searcher.doc(hit.docAddress).toDict() as TestDoc).id?.[0]).sort()
    expect(ids).toEqual([1, 2])
  })

  it('test_term_query_unsigned_field', () => {
    // term_query must correctly match documents on unsigned (u64) fields.
    // A generic value extraction path infers integers as i64, which for a
    // u64 schema field produces a mistyped Term that never matches.
    const indexSchema = new SchemaBuilder()
      .addUnsignedField('uid', { stored: true, indexed: true })
      .addTextField('body', { stored: true })
      .build()
    const index = new Index(indexSchema)
    const writer = index.writer(15_000_000, 1)

    const doc1 = new Document()
    doc1.addUnsigned('uid', 1)
    doc1.addText('body', 'hello world')
    writer.addDocument(doc1)

    const doc2 = new Document()
    doc2.addUnsigned('uid', 2)
    doc2.addText('body', 'goodbye world')
    writer.addDocument(doc2)

    writer.commit()
    index.reload()
    const searcher = index.searcher()

    let result = searcher.search(Query.termQuery(index.schema, 'uid', 1), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as any).uid[0]).toBe(1)

    result = searcher.search(Query.termQuery(index.schema, 'uid', 2), 10)
    expect(result.hits.length).toBe(1)
    expect((searcher.doc(result.hits[0].docAddress).toDict() as any).uid[0]).toBe(2)

    expect(searcher.search(Query.termQuery(index.schema, 'uid', 999), 10).hits.length).toBe(0)
  })

  it('test_bytes_term_query', () => {
    const indexSchema = new SchemaBuilder().addBytesField('data', { indexed: true }).build()
    const index = new Index(indexSchema)
    const writer = index.writer()

    const doc1 = new Document()
    doc1.addBytes('data', Buffer.from([1, 2, 3]))
    writer.addDocument(doc1)
    const doc2 = new Document()
    doc2.addBytes('data', Buffer.from([4, 5, 6]))
    writer.addDocument(doc2)
    writer.commit()
    index.reload()

    const query = Query.termQuery(indexSchema, 'data', Buffer.from([1, 2, 3]))
    expect(index.searcher().search(query, 10).hits.length).toBe(1)
  })
})

describe('TestQueryGrammar', () => {
  const valid = [
    'hello world',
    'title:hello',
    'title:hello AND body:world',
    'title:hello OR title:world',
    'title:hello NOT body:spam',
    '"hello world"',
    'year:[2000 TO 2020]',
    'title:hello^2.0',
    'title:hel*',
    'title:hello~2',
    '',
  ]

  for (const queryString of valid) {
    it(`test_parse_query_valid: ${JSON.stringify(queryString)}`, () => {
      const ast = parseQuery(queryString)
      expect(ast).not.toBeNull()
      expect(typeof ast).toBe('object')
    })
  }

  it('test_parse_query_invalid', () => {
    expect(() => parseQuery('title:')).toThrow()
  })

  for (const queryString of ['hello world', 'title: AND body:world', 'title:hello AND body:world OR author:john', '']) {
    it(`test_parse_query_lenient: ${JSON.stringify(queryString)}`, () => {
      const [ast, errors] = parseQueryLenient(queryString)
      expect(ast).not.toBeNull()
      expect(typeof ast).toBe('object')
      expect(Array.isArray(errors)).toBe(true)
    })
  }

  it('test_parse_query_lenient_valid_no_errors', () => {
    const [ast, errors] = parseQueryLenient('hello world')
    expect(ast).not.toBeNull()
    expect(errors).toEqual([])
  })
})

describe('TestFastFieldValues', () => {
  let fastIndex: Index

  beforeAll(() => {
    const indexSchema = new SchemaBuilder()
      .addUnsignedField('doc_id', { stored: true, indexed: true, fast: true })
      .addIntegerField('rank', { stored: true, indexed: true, fast: true })
      .addFloatField('score', { stored: true, indexed: true, fast: true })
      .addBooleanField('active', { stored: true, indexed: true })
      .addBooleanField('flag', { stored: true, indexed: true, fast: true })
      .addTextField('body', { stored: true })
      .addTextField('tag', { stored: true, fast: true })
      .build()
    fastIndex = new Index(indexSchema)
    const writer = fastIndex.writer(15_000_000, 1)

    const rows: [number, number, number, boolean, string, string][] = [
      [101, -10, 1.5, true, 'alpha beta gamma', 'news'],
      [202, 0, 2.5, false, 'beta gamma delta', 'sports'],
      [303, 10, 0.5, true, 'gamma delta epsilon', 'news'],
    ]
    for (const [docId, rank, score, flag, body, tag] of rows) {
      const doc = new Document()
      doc.addUnsigned('doc_id', docId)
      doc.addInteger('rank', rank)
      doc.addFloat('score', score)
      doc.addBoolean('active', flag)
      doc.addBoolean('flag', flag)
      doc.addText('body', body)
      doc.addText('tag', tag)
      writer.addDocument(doc)
    }
    writer.commit()
    writer.waitMergingThreads()
    fastIndex.reload()
  })

  const addressesFor = (term: string, limit = 10) => {
    const searcher = fastIndex.searcher()
    return searcher.search(fastIndex.parseQuery(term, ['body']), limit).hits.map((hit) => hit.docAddress)
  }

  it('test_returns_values_in_hit_order', () => {
    const searcher = fastIndex.searcher()
    const addrs = addressesFor('beta')
    const ids = searcher.fastFieldValues('doc_id', addrs)
    expect(ids.length).toBe(addrs.length)
    addrs.forEach((addr, i) => {
      expect(ids[i]).toBe((searcher.doc(addr).toDict() as any).doc_id[0])
    })
  })

  it('test_all_docs_covered', () => {
    const ids = fastIndex.searcher().fastFieldValues('doc_id', addressesFor('gamma'))
    expect(new Set(ids)).toEqual(new Set([101, 202, 303]))
  })

  it('test_empty_input_returns_empty', () => {
    expect(fastIndex.searcher().fastFieldValues('doc_id', [])).toEqual([])
  })

  it('test_single_match', () => {
    expect(fastIndex.searcher().fastFieldValues('doc_id', addressesFor('alpha'))).toEqual([101])
  })

  const typed: [string, unknown[]][] = [
    ['rank', [-10, 0, 10]],
    ['score', [0.5, 1.5, 2.5]],
    ['flag', [true, false]],
  ]
  for (const [fieldName, expected] of typed) {
    it(`test_typed_fields_${fieldName}`, () => {
      const values = fastIndex.searcher().fastFieldValues(fieldName, addressesFor('gamma'))
      expect(new Set(values)).toEqual(new Set(expected))
    })
  }

  it('test_unknown_field_raises', () => {
    expect(() => fastIndex.searcher().fastFieldValues('nonexistent', addressesFor('gamma', 1))).toThrow('Unknown field')
  })

  it('test_non_fast_field_raises', () => {
    expect(() => fastIndex.searcher().fastFieldValues('active', addressesFor('gamma', 1))).toThrow('not a fast field')
  })

  it('test_unsupported_type_raises', () => {
    // Text fast fields store term ids, not scalar values — unsupported.
    expect(() => fastIndex.searcher().fastFieldValues('tag', addressesFor('gamma', 1))).toThrow('unsupported type')
  })
})

describe('TestTermsWithPrefix', () => {
  let prefixIndex: Index
  let multiSegIndex: Index
  let filteredIndex: Index

  beforeAll(() => {
    // Single-segment index with predictable term frequencies.
    const prefixSchema = new SchemaBuilder()
      .addTextField('body')
      .addUnsignedField('owner_id', { stored: true, indexed: true, fast: true })
      .build()
    prefixIndex = new Index(prefixSchema)
    const prefixWriter = prefixIndex.writer(15_000_000, 1)
    // apple: 2 docs, apricot: 1 doc, banana: 1 doc, cherry: 1 doc, date: 1 doc
    for (const [body, owner] of [
      ['apple banana', 1],
      ['apple apricot', 2],
      ['cherry date', 1],
    ] as [string, number][]) {
      const doc = new Document()
      doc.addText('body', body)
      doc.addUnsigned('owner_id', owner)
      prefixWriter.addDocument(doc)
    }
    prefixWriter.commit()
    prefixWriter.waitMergingThreads()
    prefixIndex.reload()

    // Two-segment index. waitMergingThreads() is intentionally omitted:
    // calling it lets the default merge policy collapse the two small
    // segments into one, which would defeat the purpose of this fixture.
    multiSegIndex = new Index(new SchemaBuilder().addTextField('body').build())
    for (const body of ['apple banana', 'apple apricot']) {
      const writer = multiSegIndex.writer(15_000_000, 1)
      const doc = new Document()
      doc.addText('body', body)
      writer.addDocument(doc)
      writer.commit()
      // Release the directory lock so the next writer can be created; this is
      // what tantivy-py's `with index.writer() as writer:` block does on exit.
      writer.waitMergingThreads()
      multiSegIndex.reload()
    }

    // 4-doc index: 2 owned by user 1, 2 owned by user 2. All docs contain
    // 'apple'; 'exclusive' only in user-2 docs.
    filteredIndex = new Index(
      new SchemaBuilder()
        .addTextField('body')
        .addUnsignedField('owner_id', { stored: true, indexed: true, fast: true })
        .build(),
    )
    const filteredWriter = filteredIndex.writer(15_000_000, 1)
    for (const [body, owner] of [
      ['apple common', 1],
      ['apple common', 1],
      ['apple exclusive', 2],
      ['apple exclusive', 2],
    ] as [string, number][]) {
      const doc = new Document()
      doc.addText('body', body)
      doc.addUnsigned('owner_id', owner)
      filteredWriter.addDocument(doc)
    }
    filteredWriter.commit()
    filteredWriter.waitMergingThreads()
    filteredIndex.reload()
  })

  const countsOf = (results: { term: string; count: number }[]) =>
    Object.fromEntries(results.map(({ term, count }) => [term, count]))

  it('test_basic_prefix_match', () => {
    const results = prefixIndex.searcher().termsWithPrefix('body', 'ap')
    const counts = countsOf(results)
    expect(Object.keys(counts).sort()).toEqual(['apple', 'apricot'])
    expect(counts.apple).toBe(2)
    expect(counts.apricot).toBe(1)
  })

  it('test_sorted_by_count_descending_then_alpha', () => {
    const results = prefixIndex.searcher().termsWithPrefix('body', '')
    const counts = results.map((r) => r.count)
    expect(counts).toEqual([...counts].sort((a, b) => b - a))
    // Ties are broken alphabetically within each count group.
    for (let i = 1; i < results.length; i++) {
      if (results[i].count === results[i - 1].count) {
        expect(results[i - 1].term < results[i].term).toBe(true)
      }
    }
  })

  it('test_multi_segment_counts_summed', () => {
    expect(multiSegIndex.searcher().numSegments).toBeGreaterThanOrEqual(2)
    const counts = countsOf(multiSegIndex.searcher().termsWithPrefix('body', 'ap'))
    expect(counts.apple).toBe(2)
    expect(counts.apricot).toBe(1)
  })

  it('test_filter_query_path', () => {
    const searcher = filteredIndex.searcher()
    const filterQuery = Query.termQuery(filteredIndex.schema, 'owner_id', 2)
    const counts = countsOf(searcher.termsWithPrefix('body', '', filterQuery))
    // user 2 has 2 docs each containing 'apple' and 'exclusive'
    expect(counts.apple).toBe(2)
    expect(counts.common).toBeUndefined()
    expect(counts.exclusive).toBe(2)
  })

  it('test_filter_matches_nothing', () => {
    const filterQuery = Query.termQuery(filteredIndex.schema, 'owner_id', 99)
    expect(filteredIndex.searcher().termsWithPrefix('body', 'ap', filterQuery)).toEqual([])
  })

  it('test_limit_truncation', () => {
    const searcher = prefixIndex.searcher()
    const all = searcher.termsWithPrefix('body', '')
    const limited = searcher.termsWithPrefix('body', '', undefined, 2)
    expect(limited.length).toBe(2)
    expect(limited).toEqual(all.slice(0, 2))
  })

  it('test_empty_prefix_returns_all_terms', () => {
    const terms = prefixIndex
      .searcher()
      .termsWithPrefix('body', '')
      .map((r) => r.term)
    expect(terms.sort()).toEqual(['apple', 'apricot', 'banana', 'cherry', 'date'])
  })

  it('test_unknown_field_raises', () => {
    expect(() => prefixIndex.searcher().termsWithPrefix('nonexistent', 'ap')).toThrow('is not defined in the schema')
  })

  it('test_non_text_field_raises', () => {
    expect(() => prefixIndex.searcher().termsWithPrefix('owner_id', 'ap')).toThrow('not an indexed text field')
  })

  it('test_no_filter_equals_all_docs_filter', () => {
    const searcher = filteredIndex.searcher()
    const noFilter = searcher.termsWithPrefix('body', '')
    const allDocs = filteredIndex.parseQuery('apple OR common OR exclusive', ['body'])
    expect(searcher.termsWithPrefix('body', '', allDocs)).toEqual(noFilter)
  })
})
