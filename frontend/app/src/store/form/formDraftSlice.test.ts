import { describe, it, expect, beforeEach } from 'vitest'
import useStore from '../store'
import type { FormDraftKey, FormDraftState } from './formDraftSlice'

interface TestClusterForm {
  name?: string
  address?: string
  treeIds?: number[]
}

interface TestTreeForm {
  latitude?: number
  longitude?: number
  plantingYear?: number
  species?: string
}

interface TestWateringPlanForm {
  date?: string
  transporterId?: number
  clusterIds?: number[]
}

const getDraft = <T>(key: FormDraftKey): FormDraftState<T> | undefined => {
  return useStore.getState().formDrafts[key] as FormDraftState<T> | undefined
}

describe('formDraftSlice', () => {
  beforeEach(() => {
    useStore.getState().clearAllFormDrafts()
  })

  describe('setFormDraft', () => {
    it('should store data with hasChanges=false', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Test', treeIds: [] })

      const draft = useStore.getState().formDrafts['cluster-create']
      expect(draft?.data).toEqual({ name: 'Test', treeIds: [] })
      expect(draft?.hasChanges).toBe(false)
    })

    it('should overwrite existing draft with hasChanges=false', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'First' })
      useStore.getState().markFormDraftChanged('cluster-create')
      useStore.getState().setFormDraft('cluster-create', { name: 'Second' })

      const draft = useStore.getState().formDrafts['cluster-create']
      expect(draft?.data).toEqual({ name: 'Second' })
      expect(draft?.hasChanges).toBe(false)
    })
  })

  describe('updateFormDraft', () => {
    it('should update data and set hasChanges=true', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Test', treeIds: [] })
      useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
        ...prev,
        treeIds: [1, 2, 3],
      }))

      const draft = getDraft<TestClusterForm>('cluster-create')
      expect(draft?.data?.treeIds).toEqual([1, 2, 3])
      expect(draft?.hasChanges).toBe(true)
    })

    it('should handle null prev value gracefully', () => {
      useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
        name: prev?.name ?? '',
        treeIds: [1, 2, 3],
      }))

      const draft = getDraft<TestClusterForm>('cluster-create')
      expect(draft?.data).toEqual({ name: '', treeIds: [1, 2, 3] })
      expect(draft?.hasChanges).toBe(true)
    })

    it('should preserve other fields when updating specific field', () => {
      useStore.getState().setFormDraft('cluster-create', {
        name: 'Test Cluster',
        address: 'Test Address',
        treeIds: [],
      })

      useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
        ...prev,
        treeIds: [1, 2, 3],
      }))

      const draft = getDraft<TestClusterForm>('cluster-create')
      expect(draft?.data?.name).toBe('Test Cluster')
      expect(draft?.data?.address).toBe('Test Address')
      expect(draft?.data?.treeIds).toEqual([1, 2, 3])
    })
  })

  describe('markFormDraftChanged', () => {
    it('should set hasChanges=true without modifying data', () => {
      useStore.getState().setFormDraft('tree-update', { latitude: 54.0, longitude: 9.0 })
      useStore.getState().markFormDraftChanged('tree-update')

      const draft = getDraft<TestTreeForm>('tree-update')
      expect(draft?.hasChanges).toBe(true)
      expect(draft?.data?.latitude).toBe(54.0)
    })

    it('should create entry with hasChanges=true if not exists', () => {
      useStore.getState().markFormDraftChanged('wateringplan-create')

      expect(useStore.getState().formDrafts['wateringplan-create']?.hasChanges).toBe(true)
      expect(useStore.getState().formDrafts['wateringplan-create']?.data).toBeNull()
    })
  })

  describe('clearFormDraft', () => {
    it('should remove draft completely', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Test' })
      useStore.getState().clearFormDraft('cluster-create')

      expect(useStore.getState().formDrafts['cluster-create']).toBeUndefined()
    })

    it('should not affect other drafts', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Create' })
      useStore.getState().setFormDraft('cluster-update', { name: 'Update' })
      useStore.getState().clearFormDraft('cluster-create')

      expect(useStore.getState().formDrafts['cluster-create']).toBeUndefined()
      expect(useStore.getState().formDrafts['cluster-update']?.data).toEqual({ name: 'Update' })
    })
  })

  describe('clearAllFormDrafts', () => {
    it('should remove all drafts', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Cluster' })
      useStore.getState().setFormDraft('tree-create', { species: 'Oak' })
      useStore.getState().setFormDraft('wateringplan-update', { date: '2024-01-01' })

      useStore.getState().clearAllFormDrafts()

      expect(useStore.getState().formDrafts).toEqual({})
    })
  })

  describe('isolation between keys', () => {
    it('should keep create and update drafts separate', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'New Cluster' })
      useStore.getState().setFormDraft('cluster-update', { name: 'Existing Cluster' })

      expect(getDraft<TestClusterForm>('cluster-create')?.data?.name).toBe('New Cluster')
      expect(getDraft<TestClusterForm>('cluster-update')?.data?.name).toBe('Existing Cluster')
    })

    it('should keep different form types separate', () => {
      useStore.getState().setFormDraft('tree-create', { species: 'Oak' })
      useStore.getState().setFormDraft('cluster-create', { name: 'Cluster' })
      useStore.getState().setFormDraft('wateringplan-create', { date: '2024-01-01' })

      expect(getDraft<TestTreeForm>('tree-create')?.data).toEqual({ species: 'Oak' })
      expect(getDraft<TestClusterForm>('cluster-create')?.data).toEqual({ name: 'Cluster' })
      expect(getDraft<TestWateringPlanForm>('wateringplan-create')?.data).toEqual({
        date: '2024-01-01',
      })
    })

    it('should allow hasChanges to differ between keys', () => {
      useStore.getState().setFormDraft('cluster-create', { name: 'Test' })
      useStore.getState().setFormDraft('cluster-update', { name: 'Test' })
      useStore.getState().markFormDraftChanged('cluster-create')

      expect(useStore.getState().formDrafts['cluster-create']?.hasChanges).toBe(true)
      expect(useStore.getState().formDrafts['cluster-update']?.hasChanges).toBe(false)
    })
  })
})

describe('Form → Map → Form roundtrip (Bug Prevention)', () => {
  beforeEach(() => {
    useStore.getState().clearAllFormDrafts()
  })

  describe('Map saves when no prior draft exists (edit flow)', () => {
    it('should save treeIds when no prior draft exists', () => {
      expect(useStore.getState().formDrafts['cluster-update']).toBeUndefined()

      useStore.getState().updateFormDraft<TestClusterForm>('cluster-update', (prev) => ({
        ...(prev ?? ({} as TestClusterForm)),
        treeIds: [101, 102, 103],
      }))

      const draft = getDraft<TestClusterForm>('cluster-update')
      expect(draft?.data?.treeIds).toEqual([101, 102, 103])
      expect(draft?.hasChanges).toBe(true)
    })

    it('should save clusterIds when no prior draft exists', () => {
      expect(useStore.getState().formDrafts['wateringplan-update']).toBeUndefined()

      useStore.getState().updateFormDraft<TestWateringPlanForm>('wateringplan-update', (prev) => ({
        ...(prev ?? ({} as TestWateringPlanForm)),
        clusterIds: [10, 20, 30],
      }))

      const draft = getDraft<TestWateringPlanForm>('wateringplan-update')
      expect(draft?.data?.clusterIds).toEqual([10, 20, 30])
    })

    it('should save coordinates when no prior draft exists', () => {
      expect(useStore.getState().formDrafts['tree-update']).toBeUndefined()

      useStore.getState().updateFormDraft<TestTreeForm>('tree-update', (prev) => ({
        ...(prev ?? ({} as TestTreeForm)),
        latitude: 54.123,
        longitude: 9.456,
      }))

      const draft = getDraft<TestTreeForm>('tree-update')
      expect(draft?.data?.latitude).toBe(54.123)
      expect(draft?.data?.longitude).toBe(9.456)
    })
  })

  describe('Form saves draft before map navigation', () => {
    it('should preserve all form fields after map updates only treeIds', () => {
      useStore.getState().setFormDraft('cluster-update', {
        name: 'Existing Cluster',
        address: 'Existing Address',
        treeIds: [1, 2],
      })

      useStore.getState().updateFormDraft<TestClusterForm>('cluster-update', (prev) => ({
        ...prev,
        treeIds: [1, 2, 3, 4, 5],
      }))

      const draft = getDraft<TestClusterForm>('cluster-update')
      expect(draft?.data?.name).toBe('Existing Cluster')
      expect(draft?.data?.address).toBe('Existing Address')
      expect(draft?.data?.treeIds).toEqual([1, 2, 3, 4, 5])
    })
  })

  it('TreeCluster: treeIds should be preserved after map selection', () => {
    useStore.getState().setFormDraft('cluster-create', {
      name: 'Stadtpark Gruppe',
      address: 'Musterstraße 1',
      treeIds: [],
    })

    useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
      ...prev,
      treeIds: [101, 102, 103],
    }))

    const draft = getDraft<TestClusterForm>('cluster-create')

    expect(draft?.data?.treeIds).toEqual([101, 102, 103])
    expect(draft?.data?.name).toBe('Stadtpark Gruppe')
    expect(draft?.data?.address).toBe('Musterstraße 1')
    expect(draft?.hasChanges).toBe(true)
  })

  it('TreeCluster update: treeIds should be preserved after map selection', () => {
    useStore.getState().setFormDraft('cluster-update', {
      name: 'Existing Cluster',
      address: 'Existing Address',
      treeIds: [1, 2],
    })

    useStore.getState().updateFormDraft<TestClusterForm>('cluster-update', (prev) => ({
      ...prev,
      treeIds: [1, 2, 3, 4, 5],
    }))

    const draft = getDraft<TestClusterForm>('cluster-update')
    expect(draft?.data?.treeIds).toEqual([1, 2, 3, 4, 5])
    expect(draft?.hasChanges).toBe(true)
  })

  it('WateringPlan: clusterIds should be preserved after map selection', () => {
    useStore.getState().setFormDraft('wateringplan-create', {
      date: new Date().toISOString(),
      transporterId: 1,
      clusterIds: [],
    })

    useStore.getState().updateFormDraft<TestWateringPlanForm>('wateringplan-create', (prev) => ({
      ...prev,
      clusterIds: [10, 20, 30],
    }))

    const draft = getDraft<TestWateringPlanForm>('wateringplan-create')
    expect(draft?.data?.clusterIds).toEqual([10, 20, 30])
    expect(draft?.data?.transporterId).toBe(1)
    expect(draft?.hasChanges).toBe(true)
  })

  it('Tree: coordinates should be preserved after map edit', () => {
    useStore.getState().setFormDraft('tree-create', {
      latitude: 54.0,
      longitude: 9.0,
      plantingYear: 2024,
    })

    useStore.getState().updateFormDraft<TestTreeForm>('tree-create', (prev) => ({
      ...prev,
      latitude: 54.123,
      longitude: 9.456,
    }))

    const draft = getDraft<TestTreeForm>('tree-create')
    expect(draft?.data?.latitude).toBe(54.123)
    expect(draft?.data?.longitude).toBe(9.456)
    expect(draft?.data?.plantingYear).toBe(2024)
    expect(draft?.hasChanges).toBe(true)
  })

  it('Tree update: coordinates should be preserved after map edit', () => {
    useStore.getState().setFormDraft('tree-update', {
      latitude: 54.0,
      longitude: 9.0,
      species: 'Eiche',
    })

    useStore.getState().updateFormDraft<TestTreeForm>('tree-update', (prev) => ({
      ...prev,
      latitude: 54.789,
      longitude: 9.012,
    }))

    const draft = getDraft<TestTreeForm>('tree-update')
    expect(draft?.data?.latitude).toBe(54.789)
    expect(draft?.data?.longitude).toBe(9.012)
    expect(draft?.data?.species).toBe('Eiche')
    expect(draft?.hasChanges).toBe(true)
  })

  it('Multiple navigations: data should persist through multiple map visits', () => {
    useStore.getState().setFormDraft('cluster-create', {
      name: 'Test',
      treeIds: [],
    })

    useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
      ...prev,
      treeIds: [1, 2],
    }))

    useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
      ...prev,
      treeIds: [1, 2, 3, 4],
    }))

    useStore.getState().updateFormDraft<TestClusterForm>('cluster-create', (prev) => ({
      ...prev,
      treeIds: [1, 3],
    }))

    const draft = getDraft<TestClusterForm>('cluster-create')
    expect(draft?.data?.treeIds).toEqual([1, 3])
    expect(draft?.data?.name).toBe('Test')
  })
})

describe('Type safety for FormDraftKey', () => {
  beforeEach(() => {
    useStore.getState().clearAllFormDrafts()
  })

  it('should accept all valid FormDraftKey combinations', () => {
    const validKeys: FormDraftKey[] = [
      'tree-create',
      'tree-update',
      'cluster-create',
      'cluster-update',
      'wateringplan-create',
      'wateringplan-update',
    ]

    validKeys.forEach((key) => {
      useStore.getState().setFormDraft(key, { test: true })
      expect(useStore.getState().formDrafts[key]).toBeDefined()
    })
  })
})
