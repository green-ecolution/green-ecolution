import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import ChapterToc from './ChapterToc'

describe('ChapterToc', () => {
  it('links every section by its anchor and indents subsections', () => {
    render(
      <ChapterToc
        sections={[
          { anchor: 'board', title: 'Board', level: 2 },
          { anchor: 'status', title: 'Status', level: 3 },
        ]}
      />,
    )

    expect(screen.getByRole('link', { name: 'Board' })).toHaveAttribute('href', '#board')
    expect(screen.getByRole('link', { name: 'Status' }).closest('li')).toHaveClass('pl-4')
  })

  it('renders nothing when the chapter has no sections', () => {
    const { container } = render(<ChapterToc sections={[]} />)

    expect(container).toBeEmptyDOMElement()
  })
})
