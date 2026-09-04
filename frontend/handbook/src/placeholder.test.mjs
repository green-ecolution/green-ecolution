import { describe, expect, it } from 'vitest'
import { placeholderPng } from './placeholder.mjs'

describe('placeholderPng', () => {
  it('writes a png signature', () => {
    const png = placeholderPng(8, 4)

    expect([...png.subarray(0, 8)]).toEqual([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
  })

  it('records the requested dimensions in the IHDR chunk', () => {
    const png = placeholderPng(1440, 900)
    const ihdr = png.indexOf(Buffer.from('IHDR', 'ascii'))

    expect(png.readUInt32BE(ihdr + 4)).toBe(1440)
    expect(png.readUInt32BE(ihdr + 8)).toBe(900)
    expect(png[ihdr + 12]).toBe(8)
    expect(png[ihdr + 13]).toBe(2)
  })

  it('ends with an IEND chunk', () => {
    const png = placeholderPng(8, 4)

    expect(png.subarray(-8, -4).toString('ascii')).toBe('IEND')
  })

  it('produces a deterministic image for the same size', () => {
    expect(placeholderPng(64, 32).equals(placeholderPng(64, 32))).toBe(true)
  })
})
