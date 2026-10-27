import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import TreeMasterDataCard from './TreeMasterDataCard'
import type { Tree } from '../../api/client'

const baseTree = {
  id: 'tree-1',
  species: 'Quercus robur',
  number: 'FL-001',
  latitude: 54.7937,
  longitude: 9.4469,
  updatedAt: new Date('2026-01-15T10:00:00Z'),
  provider: null,
} as unknown as Tree

const treePlantedIn = (year: number): Tree => ({ ...baseTree, plantingYear: year })

describe('TreeMasterDataCard', () => {
  it('shows the planting year', () => {
    render(<TreeMasterDataCard tree={treePlantedIn(2020)} />)

    expect(screen.getByText('2020')).toBeInTheDocument()
  })

  // A tree whose planting is still ahead exists as a plan only, which explains
  // its missing watering status and why no sensor can be linked to it.
  it('marks a tree whose planting year has not arrived yet', () => {
    const year = new Date().getFullYear() + 3
    render(<TreeMasterDataCard tree={treePlantedIn(year)} />)

    expect(screen.getByText(/noch nicht gepflanzt/i)).toBeInTheDocument()
    expect(screen.getByText(new RegExp(`für ${year} geplant`, 'i'))).toBeInTheDocument()
  })

  it('leaves a tree that has been planted unmarked', () => {
    render(<TreeMasterDataCard tree={treePlantedIn(2020)} />)

    expect(screen.queryByText(/noch nicht gepflanzt/i)).not.toBeInTheDocument()
  })
})
