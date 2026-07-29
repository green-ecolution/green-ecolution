import { describe, it, expect, afterEach } from 'vitest'
import { render, cleanup } from '@testing-library/react'
import { Lock } from 'lucide-react'
import { AlertIcon } from './alert'

afterEach(cleanup)

describe('AlertIcon', () => {
  it('falls back to the variant glyph', () => {
    const { container } = render(<AlertIcon variant="warning" />)
    expect(container.querySelector('svg')).toHaveClass('text-yellow')
  })

  it('renders the supplied icon instead', () => {
    const { container } = render(<AlertIcon variant="warning" icon={Lock} />)
    const svg = container.querySelector('svg')
    expect(svg).toHaveClass('text-yellow')
    // lucide sets the icon name as a class, which is how we tell the glyphs apart
    expect(svg?.getAttribute('class')).toMatch(/lucide-lock/)
  })
})
