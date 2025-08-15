import { TextAnalyzerBuilder, TokenizerStatic, FilterStatic, Index, SchemaBuilder } from '../index'

const myAnalyzer = new TextAnalyzerBuilder(
  // Create a `Tokenizer` instance.
  // It instructs the builder about which type of tokenizer
  // to create internally and with which arguments.
  TokenizerStatic.regex('(?i)([a-z]+)'),
)
  .filter(
    // Create a `Filter` instance.
    // Like `Tokenizer`, this object provides instructions
    // to the builder.
    FilterStatic.lowercase(),
  )
  .filter(
    // Define custom words.
    FilterStatic.customStopword(['www', 'com']),
  )
  // Finally, build a TextAnalyzer
  // chaining all tokenizer > [filter, ...] steps together.
  .build()

// We can check that our new analyzer is working as expected
// by passing some text to its `.analyze()` method.
const tokens = myAnalyzer.analyze('www.this1website1might1exist.com')
console.log('Analyzed tokens:', tokens)
// Will print: ['this', 'website', 'might', 'exist']

// The next step is to register our analyzer with an index.
const schema = new SchemaBuilder().addTextField('content', { tokenizerName: 'custom_analyzer' }).build()

const index = new Index(schema)
index.registerTokenizer('custom_analyzer', myAnalyzer)

// Validate the results
console.assert(Array.isArray(tokens), 'Tokens should be an array')
console.assert(tokens.length > 0, 'Tokens array should not be empty')
console.assert(tokens.includes('this'), 'Tokens should include "this"')
console.assert(tokens.includes('website'), 'Tokens should include "website"')
console.assert(!tokens.includes('www'), 'Tokens should not include stop word "www"')
console.assert(!tokens.includes('com'), 'Tokens should not include stop word "com"')
