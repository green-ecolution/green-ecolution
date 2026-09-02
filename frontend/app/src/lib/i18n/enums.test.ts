import { expect, it } from 'vitest'
import {
  DataHealth,
  DrivingLicense,
  SensorStatus,
  SoilCondition,
  UserStatus,
  VehicleStatus,
  VehicleType,
  WateringPlanStatus,
  WateringStatus,
} from '@green-ecolution/backend-client'
import de from '@/locales/de/enums.json'
import en from '@/locales/en/enums.json'

function flatten(value: unknown, prefix = ''): string[] {
  if (typeof value === 'string') return [prefix]
  if (value === null || typeof value !== 'object') return []
  return Object.entries(value).flatMap(([key, child]) =>
    flatten(child, prefix ? `${prefix}.${key}` : key),
  )
}

/** Same traversal as `flatten`, but keeps the leaf string alongside its key. */
function flattenEntries(value: unknown, prefix = ''): [string, string][] {
  if (typeof value === 'string') return [[prefix, value]]
  if (value === null || typeof value !== 'object') return []
  return Object.entries(value).flatMap(([key, child]) =>
    flattenEntries(child, prefix ? `${prefix}.${key}` : key),
  )
}

/** Every catalog leaf, tagged with its language, for checks that scan both. */
function taggedEntries(): [string, string][] {
  return [
    ...flattenEntries(de).map(([key, value]): [string, string] => [`de:${key}`, value]),
    ...flattenEntries(en).map(([key, value]): [string, string] => [`en:${key}`, value]),
  ]
}

const PLACEHOLDER = /\{[a-zA-Z0-9_]+\}/g

function placeholdersIn(value: string): Set<string> {
  return new Set(value.match(PLACEHOLDER) ?? [])
}

it('has the same key set in German and English', () => {
  expect(flatten(en).sort()).toEqual(flatten(de).sort())
})

// A group whose catalog entries don't map 1:1 onto a backend-client enum
// (e.g. `dataHealth`'s derived UI levels, or a group's local `unknown`
// sentinel outside the backend's own values) is intentionally not asserted
// here beyond the enum values it does share with the backend.
const ENUM_GROUPS: { group: string; values: readonly string[]; hasDescription: boolean }[] = [
  { group: 'wateringStatus', values: Object.values(WateringStatus), hasDescription: true },
  { group: 'sensorStatus', values: Object.values(SensorStatus), hasDescription: true },
  { group: 'userStatus', values: Object.values(UserStatus), hasDescription: false },
  { group: 'vehicleStatus', values: Object.values(VehicleStatus), hasDescription: true },
  { group: 'vehicleType', values: Object.values(VehicleType), hasDescription: false },
  { group: 'wateringPlanStatus', values: Object.values(WateringPlanStatus), hasDescription: true },
  { group: 'dataHealth', values: Object.values(DataHealth), hasDescription: true },
  { group: 'drivingLicense', values: Object.values(DrivingLicense), hasDescription: false },
  { group: 'soilCondition', values: Object.values(SoilCondition), hasDescription: false },
]

it('has a German label (and description, where the group carries one) for every backend enum value', () => {
  const deKeys = flatten(de)
  const missing = ENUM_GROUPS.flatMap(({ group, values, hasDescription }) =>
    values.flatMap((value) =>
      (hasDescription ? ['label', 'description'] : ['label'])
        .map((suffix) => `${group}.${value}.${suffix}`)
        .filter((key) => !deKeys.includes(key)),
    ),
  )
  expect(missing).toEqual([])
})

it('uses the same placeholders in German and English for every key', () => {
  const deEntries = new Map(flattenEntries(de))
  const enEntries = new Map(flattenEntries(en))
  const mismatches: string[] = []
  for (const [key, deValue] of deEntries) {
    const enValue = enEntries.get(key)
    // A key missing from one catalog is reported by the key-set-parity test.
    if (enValue === undefined) continue
    const deSet = placeholdersIn(deValue)
    const enSet = placeholdersIn(enValue)
    const same = deSet.size === enSet.size && [...deSet].every((p) => enSet.has(p))
    if (!same) {
      mismatches.push(`${key}: de=${[...deSet].sort().join(',')} en=${[...enSet].sort().join(',')}`)
    }
  }
  expect(mismatches).toEqual([])
})

it('has no empty or whitespace-only values', () => {
  const blank = taggedEntries()
    .filter(([, value]) => value.trim().length === 0)
    .map(([key]) => key)
  expect(blank).toEqual([])
})

it('has no leftover template-literal syntax from the old renderer form', () => {
  const leftover = taggedEntries()
    .filter(([, value]) => value.includes('${'))
    .map(([key]) => key)
  expect(leftover).toEqual([])
})

it('uses single-brace interpolation only', () => {
  const doubled = taggedEntries()
    .filter(([, value]) => value.includes('{{') || value.includes('}}'))
    .map(([key]) => key)
  expect(doubled).toEqual([])
})
