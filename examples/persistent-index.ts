import { SchemaBuilder, Index } from '../index'
import * as os from 'os'
import * as path from 'path'
import * as fs from 'fs'

const schemaBuilder = new SchemaBuilder()
schemaBuilder.addTextField('title', { stored: true })
schemaBuilder.addTextField('body', { stored: true })
schemaBuilder.addIntegerField('doc_id', { stored: true })
const schema = schemaBuilder.build()

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tantivy-'))
const indexPath = path.join(tmpDir, 'index')
fs.mkdirSync(indexPath, { recursive: true })
const persistentIndex = new Index(schema, indexPath)

// Assertions
console.assert(fs.existsSync(indexPath), 'Index directory should exist')

const writer = persistentIndex.writer()
console.assert(writer !== undefined, 'Writer should be created for persistent index')

// Cleanup
try {
  fs.rmSync(tmpDir, { recursive: true, force: true })
} catch {}
