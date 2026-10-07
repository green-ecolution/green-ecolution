import { describe, expect, it } from 'vitest'
import { DataHealth, SensorStatus } from '@green-ecolution/backend-client'
import { legacyPreparedRedirect, statusesForView } from './sensorListView'

describe('statusesForView', () => {
  it('hides prepared sensors in the activated view when no status is picked', () => {
    expect(statusesForView('activated', undefined)).toEqual([
      SensorStatus.Online,
      SensorStatus.Offline,
    ])
  })

  it('narrows the activated view to the picked statuses', () => {
    expect(statusesForView('activated', [SensorStatus.Offline])).toEqual([SensorStatus.Offline])
  })

  it('never lets a picked prepared status leak into the activated view', () => {
    expect(statusesForView('activated', [SensorStatus.Prepared])).toEqual([
      SensorStatus.Online,
      SensorStatus.Offline,
    ])
  })

  it('shows only prepared sensors in the prepared view, ignoring picked statuses', () => {
    expect(statusesForView('prepared', [SensorStatus.Online])).toEqual([SensorStatus.Prepared])
  })
})

describe('legacyPreparedRedirect', () => {
  it('leaves searches without a prepared status alone', () => {
    expect(legacyPreparedRedirect({ page: 1, statuses: [SensorStatus.Online] })).toBeUndefined()
    expect(legacyPreparedRedirect({ page: 1 })).toBeUndefined()
  })

  it('leaves the prepared view alone', () => {
    expect(
      legacyPreparedRedirect({ page: 1, view: 'prepared', statuses: [SensorStatus.Prepared] }),
    ).toBeUndefined()
  })

  it('moves a prepared-only status filter to the prepared view', () => {
    expect(
      legacyPreparedRedirect({
        page: 2,
        q: 'eui',
        statuses: [SensorStatus.Prepared],
        hasTree: false,
        dataHealth: [DataHealth.Ok],
        clusterIds: ['cluster-1'],
      }),
    ).toEqual({
      page: 1,
      q: 'eui',
      view: 'prepared',
      statuses: undefined,
      hasTree: undefined,
      dataHealth: undefined,
      clusterIds: undefined,
    })
  })

  it('drops prepared from a mixed status filter and stays in the activated view', () => {
    expect(
      legacyPreparedRedirect({ page: 3, statuses: [SensorStatus.Prepared, SensorStatus.Online] }),
    ).toEqual({ page: 3, statuses: [SensorStatus.Online] })
  })
})
