import * as fs from 'fs'
import { marked } from 'marked'
import * as path from 'path'

/**
 * Extract example references from markdown
 * Looks for: <!-- example:name source:path/to/file.ts -->
 */
function extractExampleRefs(markdownContent: string): Array<{ name: string; sourceFile: string }> {
  const refs: Array<{ name: string; sourceFile: string }> = []
  const tokens = marked.lexer(markdownContent)

  for (const token of tokens) {
    if (token.type === 'html') {
      const match = token.raw.match(/<!--\s*example:([\w-]+)\s+source:([^\s]+)\s*-->/)
      if (match) {
        refs.push({ name: match[1], sourceFile: match[2] })
      }
    }
  }
  return refs
}

/**
 * Update markdown with source file contents
 */
export function syncFromSource(markdownFile: string): { updated: boolean; changes: string[] } {
  const markdownPath = path.resolve(markdownFile)
  const markdownDir = path.dirname(markdownPath)
  let content = fs.readFileSync(markdownPath, 'utf-8')
  const originalContent = content

  const refs = extractExampleRefs(content)
  const changes: string[] = []

  for (const ref of refs) {
    const sourceFilePath = path.resolve(markdownDir, ref.sourceFile)

    if (!fs.existsSync(sourceFilePath)) {
      throw new Error(`Source file not found: ${ref.sourceFile}`)
    }

    // Read the source file, replace the import path, and remove trailing newlines
    const sourceContent = fs
      .readFileSync(sourceFilePath, 'utf-8')
      .replace("'../index'", "'@oxdev03/node-tantivy-binding'")
      .replace(/[\r\n]+$/, '')
    const commentTag = `<!-- example:${ref.name} source:${ref.sourceFile} -->`

    // Find and replace the code block after the comment
    const regex = new RegExp(
      `(${commentTag.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})(\n\n)\`\`\`typescript[\\s\\S]*?\`\`\``,
      'g',
    )

    const newContent = content.replace(regex, `$1$2\`\`\`typescript\n${sourceContent}\n\`\`\``)
    if (newContent !== content) {
      changes.push(`Updated ${ref.name} from ${ref.sourceFile}`)
      content = newContent
    }
  }

  const updated = content !== originalContent
  if (updated) {
    fs.writeFileSync(markdownPath, content, 'utf-8')
  }

  return { updated, changes }
}
