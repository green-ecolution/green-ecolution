import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { renderHook } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import { DataHealth } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import {
  dataQualityLevel,
  hasQualityWarning,
  useDataQualityDetails,
  useQualityReasonLabel,
} from './useDetailsForDataHealth'

const ok = { dataHealth: DataHealth.Ok, implausibleRecent: 0 }
const flagged = { dataHealth: DataHealth.Ok, implausibleRecent: 2 }
const suspect = { dataHealth: DataHealth.Suspect, implausibleRecent: 9 }

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nextProvider i18n={getI18n()}>{children}</I18nextProvider>
)

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

describe('useDataQualityDetails', () => {
  it('never calls flagged data plausible', () => {
    const { result } = renderHook(() => useDataQualityDetails(), { wrapper })
    expect(result.current(flagged).label).not.toBe(result.current(ok).label)
    expect(result.current(flagged).color).not.toBe(result.current(ok).color)
  })

  it('labels the three levels distinctly', () => {
    const { result } = renderHook(() => useDataQualityDetails(), { wrapper })
    expect(result.current(ok).label).toBe('Daten plausibel')
    expect(result.current(flagged).label).toBe('Einzelne Werte verworfen')
    expect(result.current(suspect).label).toBe('Datenqualität prüfen')
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

describe('useQualityReasonLabel', () => {
  it('translates the known reason codes', () => {
    const { result } = renderHook(() => useQualityReasonLabel(), { wrapper })
    expect(result.current('out_of_range')).toBe('Wert außerhalb des möglichen Bereichs')
    expect(result.current('implausible_jump')).toBe('Unplausibler Sprung gegenüber dem Vorwert')
  })

  it('falls back for an unknown code', () => {
    const { result } = renderHook(() => useQualityReasonLabel(), { wrapper })
    expect(result.current('above_soil_capacity')).toBe('Unbekannter Grund')
  })
})
