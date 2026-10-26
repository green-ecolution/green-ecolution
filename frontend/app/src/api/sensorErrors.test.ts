import { describe, expect, it } from 'vitest'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { mapActivateError, mapReassignError, mapDeactivateError } from './sensorErrors'

const responseError = (status: number) => new Response(null, { status })

describe('sensorErrors', () => {
  it('maps 404 on activate', () => {
    expect(mapActivateError(responseError(404))).toMatch(/existiert nicht/i)
  })
  it('maps 409 on activate', () => {
    expect(mapActivateError(responseError(409))).toMatch(/bereits/i)
  })
  it('maps 409 on reassign to a target conflict', () => {
    expect(mapReassignError(responseError(409))).toBe(
      'Das Objekt ist bereits einem anderen Sensor zugeordnet.',
    )
  })
  it('lets the target module word the conflicts', () => {
    const tree = irrigationManifest.targets?.[0]
    expect(mapActivateError(responseError(409), tree)).toBe(
      'Sensor ist bereits einem Baum zugeordnet.',
    )
    expect(mapReassignError(responseError(409), tree)).toBe(
      'Der Baum ist bereits einem anderen Sensor zugeordnet.',
    )
    expect(mapReassignError(new Error('boom'), tree)).toBe(
      'Baumwechsel fehlgeschlagen. Bitte erneut versuchen.',
    )
  })
  it('names the organization mismatch on activate and reassign', () => {
    expect(mapActivateError(responseError(422))).toMatch(/Organisation/i)
    expect(mapReassignError(responseError(422))).toMatch(/Organisation/i)
  })
  it('maps unknown errors generically on deactivate', () => {
    expect(mapDeactivateError(new Error('boom'))).toMatch(/fehlgeschlagen/i)
  })
})
