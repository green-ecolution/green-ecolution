import { describe, it, expect } from 'vitest'
import { act, renderHook } from '@/test/utils'
import { usernameFromEmail, useAddMemberForm } from './addMemberForm'

describe('usernameFromEmail', () => {
  it('uses the lower-cased part before the @', () => {
    expect(usernameFromEmail('Anna.Schmidt@tbz.example')).toBe('anna.schmidt')
  })

  it('lower-cases an input without an @', () => {
    expect(usernameFromEmail('Anna.Schmidt')).toBe('anna.schmidt')
  })

  it('drops whitespace and caps the length at 64 characters', () => {
    expect(usernameFromEmail(' an na@tbz.example')).toBe('anna')
    expect(usernameFromEmail(`${'a'.repeat(80)}@tbz.example`)).toHaveLength(64)
  })
})

const fill = (result: { current: ReturnType<typeof useAddMemberForm> }) => {
  act(() => {
    result.current.setField('firstName', 'Anna')
    result.current.setField('lastName', 'Schmidt')
    result.current.setField('email', 'anna.schmidt@tbz.example')
  })
}

describe('useAddMemberForm', () => {
  it('starts empty and invalid without showing errors', () => {
    const { result } = renderHook(() => useAddMemberForm())
    expect(result.current.valid).toBe(false)
    expect(result.current.errors).toEqual({})
  })

  it('derives the username from the e-mail until it is edited by hand', () => {
    const { result } = renderHook(() => useAddMemberForm())
    fill(result)
    expect(result.current.values.username).toBe('anna.schmidt')

    act(() => result.current.setField('username', 'aschmidt'))
    act(() => result.current.setField('email', 'someone.else@tbz.example'))

    expect(result.current.values.username).toBe('aschmidt')
  })

  it('is valid with all four fields filled', () => {
    const { result } = renderHook(() => useAddMemberForm())
    fill(result)
    expect(result.current.valid).toBe(true)
    expect(result.current.errors).toEqual({})
  })

  it('reports a message at each invalid field', () => {
    const { result } = renderHook(() => useAddMemberForm())
    act(() => {
      result.current.setField('firstName', '  ')
      result.current.setField('lastName', '')
      result.current.setField('email', 'not-an-email')
      result.current.setField('username', '')
    })
    expect(result.current.valid).toBe(false)
    expect(result.current.errors.firstName).toBeTruthy()
    expect(result.current.errors.lastName).toBeTruthy()
    expect(result.current.errors.email).toBe('E-Mail-Adresse ist ungültig.')
    expect(result.current.errors.username).toBe('Benutzername ist erforderlich.')
  })

  it('resets to an empty form that derives the username again', () => {
    const { result } = renderHook(() => useAddMemberForm())
    fill(result)
    act(() => result.current.setField('username', 'aschmidt'))
    act(() => result.current.reset())
    expect(result.current.values).toEqual({
      firstName: '',
      lastName: '',
      email: '',
      username: '',
      roleIds: [],
    })
    act(() => result.current.setField('email', 'b@x.example'))
    expect(result.current.values.username).toBe('b')
  })
})
