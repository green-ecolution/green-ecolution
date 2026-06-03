import { describe, it, expect } from 'vitest'
import type { z } from 'zod'
import { Route } from './index'

const parseSearch = (input: Record<string, unknown>) =>
  (Route.options.validateSearch as unknown as z.ZodSchema<{ page: number }>).parse(input)

const loaderDeps = Route.options.loaderDeps as unknown as (opts: {
  search: Record<string, unknown>
}) => { page: number }

describe('/trees route pagination (GECO-129)', () => {
  it('parses page from search params', () => {
    expect(parseSearch({ page: 2 }).page).toBe(2)
  })

  it('falls back to page 1 when page is missing or invalid', () => {
    expect(parseSearch({}).page).toBe(1)
    expect(parseSearch({ page: 'invalid' }).page).toBe(1)
  })

  it('passes page from search params through loaderDeps', () => {
    expect(loaderDeps({ search: { page: 3 } }).page).toBe(3)
  })
})
