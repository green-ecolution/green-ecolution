import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { renderHook } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import { SensorStatus } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { useSensorStatusDetails } from './useDetailsForSensorStatus'

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nextProvider i18n={getI18n()}>{children}</I18nextProvider>
)

describe('useSensorStatusDetails', () => {
  it('carries a hex colour for every status', () => {
    const { result } = renderHook(() => useSensorStatusDetails(), { wrapper })

    for (const status of Object.values(SensorStatus)) {
      expect(result.current(status).colorHex).toMatch(/^#[0-9A-Fa-f]{6}$/)
    }
  })
})
