import { describe, expect, it } from 'vitest'
import { DataHealth, SensorStatus } from '@green-ecolution/backend-client'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'
import {
  legacyHasTreeRedirect,
  legacyPreparedRedirect,
  sensorListParams,
  sensorSearchParserFor,
  statusesForView,
} from './sensorListView'

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
        attached: false,
        dataHealth: [DataHealth.Ok],
        clusterIds: ['cluster-1'],
      }),
    ).toEqual({
      page: 1,
      q: 'eui',
      view: 'prepared',
      statuses: undefined,
      attached: undefined,
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

describe('sensorListParams', () => {
  const base = { page: 1 }

  it('passes the link filter through as attached', () => {
    expect(sensorListParams({ ...base, attached: true }).attached).toBe(true)
    expect(sensorListParams({ ...base, attached: false }).attached).toBe(false)
    expect(sensorListParams(base).attached).toBeUndefined()
  })

  it('passes the sensor ids the target filters resolved as id', () => {
    expect(sensorListParams(base, ['s1', 's2']).id).toEqual(['s1', 's2'])
  })
})

describe('legacyHasTreeRedirect', () => {
  it('moves a tree filter from an old link to the link filter', () => {
    expect(legacyHasTreeRedirect({ page: 1, hasTree: false })).toEqual({
      page: 1,
      hasTree: undefined,
      attached: false,
    })
  })

  it('leaves current links alone', () => {
    expect(legacyHasTreeRedirect({ page: 1, attached: true })).toBeUndefined()
  })
})

describe('sensorSearchParserFor', () => {
  it('keeps the search params of every module target filter', () => {
    const Nothing = () => null
    const parse = sensorSearchParserFor([
      probeTargetManifest('probe', [
        probeTarget('probe', 'probe', {
          listFilter: {
            param: 'probeIds',
            Control: Nothing,
            Chips: Nothing,
            sensorIds: (value) => ({ queryKey: ['probe', value] }),
          },
        }),
      ]),
    ])

    expect(parse({ page: 1, probeIds: ['p-1'], other: 'x' })).toEqual({
      page: 1,
      probeIds: ['p-1'],
    })
    expect(parse({ page: 1, probeIds: 'broken' })).toEqual({ page: 1 })
  })
})
