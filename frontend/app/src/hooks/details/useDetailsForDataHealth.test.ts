import { describe, expect, it } from 'vitest'
import { DataHealth } from '@green-ecolution/backend-client'
import {
  getDataHealthDetails,
  hasQualityWarning,
  qualityReasonLabel,
} from './useDetailsForDataHealth'

describe('getDataHealthDetails', () => {
  it('labels a suspect sensor', () => {
    expect(getDataHealthDetails(DataHealth.Suspect).label).toBe('Datenqualität prüfen')
  })

  it('labels a healthy sensor', () => {
    expect(getDataHealthDetails(DataHealth.Ok).label).toBe('Daten plausibel')
  })
})

describe('hasQualityWarning', () => {
  it('warns on a suspect sensor', () => {
    expect(hasQualityWarning({ dataHealth: DataHealth.Suspect, implausibleRecent: 0 })).toBe(true)
  })

  it('warns on flagged values without a suspicion', () => {
    expect(hasQualityWarning({ dataHealth: DataHealth.Ok, implausibleRecent: 2 })).toBe(true)
  })

  it('stays quiet on clean data', () => {
    expect(hasQualityWarning({ dataHealth: DataHealth.Ok, implausibleRecent: 0 })).toBe(false)
  })
})

describe('qualityReasonLabel', () => {
  it('translates the known reason codes', () => {
    expect(qualityReasonLabel('out_of_range')).toBe('Wert außerhalb des möglichen Bereichs')
    expect(qualityReasonLabel('implausible_jump')).toBe('Unplausibler Sprung gegenüber dem Vorwert')
  })

  it('falls back for an unknown code', () => {
    expect(qualityReasonLabel('above_soil_capacity')).toBe('Unbekannter Grund')
  })
})
