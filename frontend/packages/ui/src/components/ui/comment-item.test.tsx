import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi, afterEach } from 'vitest'
import { CommentItem } from './comment-item'

afterEach(cleanup)

const author = { name: 'Tom Bergmann' }
const body = 'Sensor an Baum 12 zeigt weiter niedrige Werte.'

describe('CommentItem', () => {
  it('renders the edit action only when canEdit is set', () => {
    render(<CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit />)
    expect(screen.getByRole('button', { name: 'Bearbeiten' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Löschen' })).not.toBeInTheDocument()
  })

  it('renders the delete action only when canDelete is set', () => {
    render(<CommentItem author={author} body={body} timestamp="Heute, 09:12" canDelete />)
    expect(screen.getByRole('button', { name: 'Löschen' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Bearbeiten' })).not.toBeInTheDocument()
  })

  it('renders neither action without permissions', () => {
    render(<CommentItem author={author} body={body} timestamp="Heute, 09:12" />)
    expect(screen.queryByRole('button', { name: 'Bearbeiten' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Löschen' })).not.toBeInTheDocument()
  })

  it('calls onDelete without asking for confirmation', async () => {
    const onDelete = vi.fn()
    render(
      <CommentItem
        author={author}
        body={body}
        timestamp="Heute, 09:12"
        canDelete
        onDelete={onDelete}
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(onDelete).toHaveBeenCalledTimes(1)
  })

  it('moves focus into the textarea when edit mode opens', async () => {
    render(
      <CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit onEdit={vi.fn()} />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))
    expect(screen.getByRole('textbox')).toHaveFocus()
  })

  it('restores the original text and focus when the inline edit is cancelled', async () => {
    render(
      <CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit onEdit={vi.fn()} />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))

    const textarea = screen.getByRole('textbox')
    await userEvent.clear(textarea)
    await userEvent.type(textarea, 'Ganz anderer Text')
    await userEvent.click(screen.getByRole('button', { name: 'Abbrechen' }))

    expect(screen.getByText(body)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Bearbeiten' })).toHaveFocus()
  })

  it('does not call onEdit when saving an unchanged body', async () => {
    const onEdit = vi.fn()
    render(
      <CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit onEdit={onEdit} />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(onEdit).not.toHaveBeenCalled()
    expect(screen.getByText(body)).toBeInTheDocument()
  })

  it('calls onEdit with the trimmed text when saving a changed body', async () => {
    const onEdit = vi.fn().mockResolvedValue(undefined)
    render(
      <CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit onEdit={onEdit} />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))

    const textarea = screen.getByRole('textbox')
    await userEvent.clear(textarea)
    await userEvent.type(textarea, '  Neuer Text  ')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(onEdit).toHaveBeenCalledWith('Neuer Text')
  })

  it('cannot save an empty or whitespace-only body', async () => {
    const onEdit = vi.fn()
    render(
      <CommentItem author={author} body={body} timestamp="Heute, 09:12" canEdit onEdit={onEdit} />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))

    const textarea = screen.getByRole('textbox')
    await userEvent.clear(textarea)
    await userEvent.type(textarea, '   ')

    expect(screen.getByRole('button', { name: 'Speichern' })).toBeDisabled()
    expect(onEdit).not.toHaveBeenCalled()
  })

  it('disables save and cancel while isSaving', async () => {
    render(
      <CommentItem
        author={author}
        body={body}
        timestamp="Heute, 09:12"
        canEdit
        onEdit={vi.fn()}
        isSaving
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Bearbeiten' }))
    expect(screen.getByRole('button', { name: 'Speichern' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Abbrechen' })).toBeDisabled()
  })
})
