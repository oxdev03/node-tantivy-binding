import { describe, it, expect } from 'vitest'
import * as fs from 'fs'
import * as path from 'path'
import { syncFromSource } from './sync-examples'

describe('Documentation Examples', () => {
  const examplesDir = path.join(__dirname, '../examples')
  const tutorialsPath = path.join(__dirname, '../docs/tutorials.md')

  // Dynamically find all example files
  const exampleFiles = fs.readdirSync(examplesDir).filter((file) => file.endsWith('.ts') && !file.startsWith('_'))

  it('should sync documentation with source files', () => {
    const result = syncFromSource(tutorialsPath)

    expect(result.changes, 'There should be no changes if the documentation is already up-to-date').toEqual([])
  })

  it('should find example files', () => {
    expect(exampleFiles.length).toBeGreaterThan(0)
    console.log('Found example files:', exampleFiles)
  })

  // Create a test case for each example file
  exampleFiles.forEach((filename) => {
    const exampleName = path.basename(filename, '.ts')

    it(`should execute ${exampleName} example`, async () => {
      const examplePath = path.join(examplesDir, filename)
      await import(examplePath)
    })
  })
})
