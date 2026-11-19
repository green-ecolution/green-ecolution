import { describe, it, expect } from 'vitest'
import { organizationIdOf, sectionOf } from './settingsSections'

describe('sectionOf', () => {
  it('reads the overview from the bare organization path', () => {
    expect(sectionOf('/settings/organization/amt')).toBe('overview')
    expect(sectionOf('/settings/organization/amt/')).toBe('overview')
  })

  it('reads the sub-page', () => {
    expect(sectionOf('/settings/organization/amt/members')).toBe('members')
    expect(sectionOf('/settings/organization/amt/areas')).toBe('areas')
  })

  it('maps a plugin detail to the plugin list', () => {
    expect(sectionOf('/settings/organization/amt/plugins/kataster')).toBe('plugins')
  })

  it('knows no section outside an organization', () => {
    expect(sectionOf('/settings/profile')).toBeNull()
    expect(sectionOf('/settings/organization')).toBeNull()
  })
})

describe('organizationIdOf', () => {
  it('reads the organization from the path', () => {
    expect(organizationIdOf('/settings/organization/amt/members')).toBe('amt')
  })

  it('is null on the account pages', () => {
    expect(organizationIdOf('/settings/profile')).toBeNull()
  })
})
