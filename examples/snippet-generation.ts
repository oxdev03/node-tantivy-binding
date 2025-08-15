import { SchemaBuilder, Index, Document, SnippetGenerator } from '../index'

// Setup index (from previous examples)
const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const index = new Index(schema)
const writer = index.writer()

const doc = new Document()
doc.addInteger('doc_id', 1)
doc.addText('title', 'The Old Man and the Sea')
doc.addText(
  'body',
  'He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.',
)

writer.addDocument(doc)
writer.commit()
writer.waitMergingThreads()

index.reload()
const searcher = index.searcher()

const query = index.parseQuery('fish days', ['title', 'body'])
const searchResults = searcher.search(query, 3)
const bestHit = searchResults.hits[0]
const bestDoc = searcher.doc(bestHit.docAddress)
const bestDocDict = bestDoc.toDict() as any

const hitText = bestDocDict.body[0]
console.log('Hit text:', hitText)
// Should output: "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish."

const snippetGenerator = SnippetGenerator.create(searcher, query, schema, 'body')
const snippet = snippetGenerator.snippetFromDoc(bestDoc)

// The snippet object provides the hit ranges. These are the marker
// offsets in the text that match the query.
const highlights = snippet.highlighted()
const firstHighlight = highlights[0]

console.log('First highlight start:', firstHighlight.start) // Should be 93
console.log('First highlight end:', firstHighlight.end) // Should be 97
console.log('Highlighted text:', hitText.slice(firstHighlight.start, firstHighlight.end)) // Should be "days"

// The snippet object can also generate a marked-up HTML snippet:
const htmlSnippet = snippet.toHtml()
console.log('HTML snippet:', htmlSnippet)
// Should output: "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four <b>days</b> now without taking a <b>fish</b>"

// Assertions
console.assert(highlights.length > 0, 'Should have highlights')
console.assert(firstHighlight.start === 93, 'First highlight should start at position 93')
console.assert(firstHighlight.end === 97, 'First highlight should end at position 97')
console.assert(hitText.slice(firstHighlight.start, firstHighlight.end) === 'days', 'Highlighted text should be "days"')
console.assert(htmlSnippet.includes('<b>days</b>'), 'HTML snippet should contain highlighted days')
console.assert(htmlSnippet.includes('<b>fish</b>'), 'HTML snippet should contain highlighted fish')
