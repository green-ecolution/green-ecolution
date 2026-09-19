import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { highlightMatch } from './highlightMatch'

describe('highlightMatch', () => {
  it('returns the text untouched without a query', () => {
    render(<p>{highlightMatch('Quercus robur', '')}</p>)
    expect(screen.getByText('Quercus robur')).toBeInTheDocument()
    expect(document.querySelector('mark')).toBeNull()
  })

  it('marks the match regardless of case', () => {
    render(<p data-testid="row">{highlightMatch('Quercus robur', 'quer')}</p>)
    expect(screen.getByTestId('row').textContent).toBe('Quercus robur')
    expect(document.querySelector('mark')?.textContent).toBe('Quer')
  })

  it('marks every occurrence', () => {
    render(<p>{highlightMatch('T-001 T-001', 'T-001')}</p>)
    expect(document.querySelectorAll('mark')).toHaveLength(2)
  })

  it('treats a regex metacharacter as literal text', () => {
    render(<p data-testid="row">{highlightMatch('T-00(1)', '(1)')}</p>)
    expect(screen.getByTestId('row').textContent).toBe('T-00(1)')
    expect(document.querySelector('mark')?.textContent).toBe('(1)')
  })
})
