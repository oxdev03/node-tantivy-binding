import { describe, it, expect } from 'vitest'

import { plus100 } from '../index'

describe('node-tantivy-binding', () => {
  it('sync function from native code', () => {
    const fixture = 42
    expect(plus100(fixture)).toBe(fixture + 100)
  })
})
