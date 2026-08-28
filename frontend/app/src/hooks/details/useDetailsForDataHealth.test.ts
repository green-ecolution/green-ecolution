import { describe, expect, it } from 'vitest'
import { DataHealth } from '@green-ecolution/backend-client'
import {
  dataQualityLevel,
  getDataQualityDetails,
  hasQualityWarning,
  qualityReasonLabel,
} from './useDetailsForDataHealth'

const ok = { dataHealth: DataHealth.Ok, implausibleRecent: 0 }
const flagged = { dataHealth: DataHealth.Ok, implausibleRecent: 2 }
const suspect = { dataHealth: DataHealth.Suspect, implausibleRecent: 9 }

describe('dataQualityLevel', () => {
  it('separates flagged values from a defect suspicion', () => {
    expect(dataQualityLevel(ok)).toBe('ok')
    expect(dataQualityLevel(flagged)).toBe('flagged')
    expect(dataQualityLevel(suspect)).toBe('suspect')
  })

  it('reports suspect even when the window holds no flagged value', () => {
    expect(dataQualityLevel({ dataHealth: DataHealth.Suspect, implausibleRecent: 0 })).toBe(
      'suspect',
    )
  })
})

describe('getDataQualityDetails', () => {
  it('never calls flagged data plausible', () => {
    expect(getDataQualityDetails(flagged).label).not.toBe(getDataQualityDetails(ok).label)
    expect(getDataQualityDetails(flagged).color).not.toBe(getDataQualityDetails(ok).color)
  })

  it('labels the three levels distinctly', () => {
    expect(getDataQualityDetails(ok).label).toBe('Daten plausibel')
    expect(getDataQualityDetails(flagged).label).toBe('Einzelne Werte verworfen')
    expect(getDataQualityDetails(suspect).label).toBe('Datenqualität prüfen')
  })
})

describe('hasQualityWarning', () => {
  it('warns on a suspect sensor', () => {
    expect(hasQualityWarning(suspect)).toBe(true)
  })

  it('warns on flagged values without a suspicion', () => {
    expect(hasQualityWarning(flagged)).toBe(true)
  })

  it('stays quiet on clean data', () => {
    expect(hasQualityWarning(ok)).toBe(false)
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
