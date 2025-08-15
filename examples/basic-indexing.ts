import { SchemaBuilder, Index } from '../index'

// Declaring our schema.
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

// Creating our index (in memory)
const index = new Index(schema)

// Assertions
console.assert(index !== undefined, 'Index should be created')
console.assert(schema !== undefined, 'Schema should be built')
