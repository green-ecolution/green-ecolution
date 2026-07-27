import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'

const hasPermission = vi.fn<(required: unknown) => boolean>(() => false)

vi.mock('./useHasPermission', () => ({
  useHasPermission: (required: unknown) => hasPermission(required),
}))

const { Can } = await import('./Can')

describe('Can', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    hasPermission.mockReturnValue(false)
  })

  it('renders children when permitted', () => {
    hasPermission.mockReturnValue(true)
    render(
      <Can permission={['tree:create']}>
        <button>Neuen Baum</button>
      </Can>,
    )
    expect(screen.getByRole('button', { name: 'Neuen Baum' })).toBeInTheDocument()
  })

  it('renders nothing when denied and no fallback is given', () => {
    render(
      <Can permission={['tree:create']}>
        <button>Neuen Baum</button>
      </Can>,
    )
    expect(screen.queryByRole('button', { name: 'Neuen Baum' })).not.toBeInTheDocument()
  })

  it('renders the fallback when denied', () => {
    render(
      <Can permission={['tree:create']} fallback={<span>Keine Berechtigung</span>}>
        <button>Neuen Baum</button>
      </Can>,
    )
    expect(screen.getByText('Keine Berechtigung')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Neuen Baum' })).not.toBeInTheDocument()
  })

  it('passes the requirement through to useHasPermission', () => {
    hasPermission.mockReturnValue(true)
    render(
      <Can permission={['vehicle:update']}>
        <span>ok</span>
      </Can>,
    )
    expect(hasPermission).toHaveBeenCalledWith(['vehicle:update'])
  })
})
