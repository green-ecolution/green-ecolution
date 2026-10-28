import { describe, expect, it } from 'vitest'
import { wasteRoots } from './api/roots'
import { wasteManifest } from './manifest'
import { binSensorTarget } from './sensorTargets'

describe('binSensorTarget', () => {
  it('is the waste module’s only sensor target and refreshes the bins', () => {
    expect(wasteManifest.targets).toEqual([binSensorTarget])
    expect(binSensorTarget.kind).toBe('bin')
    expect(binSensorTarget.labelKey).toBe('waste:sensor.target.bin')
    expect(binSensorTarget.invalidate).toEqual(wasteRoots.bin)
    expect(binSensorTarget.listFilter).toBeUndefined()
  })

  it('offers the full threshold as the module’s settings section', () => {
    expect(wasteManifest.settingsSection).toBeDefined()
  })
})
