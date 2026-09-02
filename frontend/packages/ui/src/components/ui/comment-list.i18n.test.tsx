import { render, screen, cleanup } from '@testing-library/react'
import { describe, expect, it, afterEach } from 'vitest'
import { UiTextProvider, fallbackTranslate, uiEn } from '../../i18n'
import { CommentList } from './comment-list'
import { CommentComposer } from './comment-composer'
import { CommentItem } from './comment-item'

afterEach(cleanup)

describe('CommentList', () => {
  it('shows the default empty hint with no children', () => {
    render(<CommentList />)
    expect(screen.getByText('Noch keine Kommentare.')).toBeInTheDocument()
  })

  it('shows an overridden empty hint', () => {
    render(<CommentList emptyLabel="Keine Beobachtungen zu dieser Gruppe." />)
    expect(screen.getByText('Keine Beobachtungen zu dieser Gruppe.')).toBeInTheDocument()
  })

  it('renders placeholders instead of the empty hint while loading', () => {
    const { container } = render(<CommentList isLoading />)
    expect(screen.queryByText('Noch keine Kommentare.')).not.toBeInTheDocument()
    expect(container.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0)
  })

  it('renders children instead of the empty hint', () => {
    render(
      <CommentList>
        <p>Ein Kommentar</p>
      </CommentList>,
    )
    expect(screen.getByText('Ein Kommentar')).toBeInTheDocument()
    expect(screen.queryByText('Noch keine Kommentare.')).not.toBeInTheDocument()
  })
})

describe('comments catalog under the English provider', () => {
  it('renders the English empty hint for CommentList', () => {
    render(
      <UiTextProvider t={fallbackTranslate(uiEn)} locale="en">
        <CommentList />
      </UiTextProvider>,
    )
    expect(screen.getByText('No comments yet.')).toBeInTheDocument()
  })

  it('renders English copy for CommentComposer', () => {
    render(
      <UiTextProvider t={fallbackTranslate(uiEn)} locale="en">
        <CommentComposer author={{ name: 'Anna Krüger' }} onSubmit={() => {}} />
      </UiTextProvider>,
    )
    expect(screen.getByRole('textbox', { name: 'Add a comment' })).toBeInTheDocument()
    expect(screen.getByRole('textbox')).toHaveAttribute('placeholder', 'Add a comment…')
    expect(screen.getByRole('button', { name: 'Comment' })).toBeInTheDocument()
  })

  it('renders English copy for CommentItem actions', () => {
    render(
      <UiTextProvider t={fallbackTranslate(uiEn)} locale="en">
        <CommentItem
          author={{ name: 'Tom Bergmann' }}
          body="Sensor reading is still low."
          timestamp="Today"
          canEdit
          canDelete
        />
      </UiTextProvider>,
    )
    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument()
  })
})
