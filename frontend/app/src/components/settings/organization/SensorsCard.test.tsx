import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@/test/utils'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import SensorsCard from './SensorsCard'
import type { SensorSettingsDraft } from './useSensorSettingsDraft'

const draftOf = (): SensorSettingsDraft => ({
  loading: false,
  loadError: null,
  dirty: false,
  valid: true,
  reset: vi.fn(),
  save: vi.fn(() => Promise.resolve()),
  server: {
    sensorOfflineAfterSecs: { value: 172_800, inheritedValue: 86_400 },
    defectStreak: { value: 3, inheritedValue: 3 },
  } as OrganizationSettingsResponse,
  draft: { offlineAfterHours: '48', defectStreak: '3' },
  issues: { offlineAfterHours: null, defectStreak: null },
  setText: vi.fn(),
})

describe('SensorsCard', () => {
  it('shows both fields with their usual values and no switch', () => {
    render(<SensorsCard draft={draftOf()} locked={false} />)
    expect(screen.getByLabelText('Sensor gilt als offline nach')).toHaveValue(48)
    expect(screen.getByLabelText('Sensor gilt als auffällig nach')).toHaveValue(3)
    expect(screen.getByText('Üblich sind 24 Stunden.')).toBeInTheDocument()
    expect(screen.getByText('Üblich sind 3 Messungen.')).toBeInTheDocument()
    expect(screen.queryByRole('switch')).toBeNull()
  })

  it('disables both inputs when locked', () => {
    render(<SensorsCard draft={draftOf()} locked />)
    expect(screen.getByLabelText('Sensor gilt als offline nach')).toBeDisabled()
    expect(screen.getByLabelText('Sensor gilt als auffällig nach')).toBeDisabled()
  })
})
