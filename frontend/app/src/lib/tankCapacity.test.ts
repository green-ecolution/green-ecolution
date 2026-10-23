import { describe, it, expect } from 'vitest'
import { tankCapacity } from './tankCapacity'

const transporter = { waterCapacity: 1000 }
const trailer = { waterCapacity: 500 }

describe('tankCapacity', () => {
  it('adds the trailer to the transporter tank', () => {
    expect(tankCapacity(transporter, trailer, true)).toBe(1500)
  })

  it('leaves out an unused transporter tank', () => {
    expect(tankCapacity(transporter, trailer, false)).toBe(500)
  })

  it('is the transporter tank without a trailer', () => {
    expect(tankCapacity(transporter, undefined, true)).toBe(1000)
  })
})
