import { bench, describe } from 'vitest'
import { SchemaBuilder, Index } from '../index'

describe('Schema and Query Performance', () => {
  bench('Create simple schema', () => {
    new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .addIntegerField('id', { stored: true })
      .build()
  })

  bench('Create complex schema with many fields', () => {
    const builder = new SchemaBuilder()
    
    // Add 50 fields of various types
    for (let i = 0; i < 10; i++) {
      builder.addTextField(`text_field_${i}`, { stored: true })
      builder.addIntegerField(`int_field_${i}`, { stored: true, indexed: true })
      builder.addFloatField(`float_field_${i}`, { stored: true })
      builder.addDateField(`date_field_${i}`, { stored: true })
      builder.addFacetField(`facet_field_${i}`)
    }
    
    builder.build()
  })

  bench('Parse simple query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .build()
    const index = new Index(schema)
    
    index.parseQuery('hello world', ['title', 'body'])
  })

  bench('Parse complex boolean query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .addTextField('category')
      .build()
    const index = new Index(schema)
    
    index.parseQuery('(title:programming OR body:coding) AND category:tech AND NOT deprecated', ['title', 'body', 'category'])
  })

  bench('Parse phrase query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .build()
    const index = new Index(schema)
    
    index.parseQuery('"machine learning algorithms"', ['title', 'body'])
  })

  bench('Parse range query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addIntegerField('year', { indexed: true })
      .addFloatField('price', { indexed: true })
      .build()
    const index = new Index(schema)
    
    index.parseQuery('year:[2020 TO 2024] AND price:[10.0 TO 100.0]', ['title'])
  })

  bench('Parse wildcard query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .build()
    const index = new Index(schema)
    
    index.parseQuery('program* AND develop* AND (test* OR debug*)', ['title', 'body'])
  })

  bench('Parse fuzzy query', () => {
    const schema = new SchemaBuilder()
      .addTextField('title', { stored: true })
      .addTextField('body')
      .build()
    const index = new Index(schema)
    
    index.parseQuery('programming~2 AND development~1', ['title', 'body'])
  })
})
