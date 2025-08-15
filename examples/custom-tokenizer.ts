import { SchemaBuilder } from '../index'

const schemaBuilderTok = new SchemaBuilder()
schemaBuilderTok.addTextField('body', { stored: true, tokenizerName: 'en_stem' })
const schema = schemaBuilderTok.build()

// Assertions
// Test that the field was added with the correct tokenizer
const schemaJson = schema.toJson()
console.assert(schemaJson.includes('body'), 'Schema should contain body field')
