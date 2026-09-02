import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi, afterEach } from 'vitest'
import { CommentComposer } from './comment-composer'

afterEach(cleanup)

const author = { name: 'Anna Krüger' }

describe('CommentComposer', () => {
  it('keeps the submit button disabled for empty input', () => {
    render(<CommentComposer author={author} onSubmit={vi.fn()} />)
    expect(screen.getByRole('button', { name: 'Kommentieren' })).toBeDisabled()
  })

  it('does not submit whitespace-only input', async () => {
    const onSubmit = vi.fn()
    render(<CommentComposer author={author} onSubmit={onSubmit} />)
    await userEvent.type(screen.getByRole('textbox'), '   ')

    const submitButton = screen.getByRole('button', { name: 'Kommentieren' })
    expect(submitButton).toBeDisabled()
    await userEvent.click(submitButton)
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('clears the field after a successful submit', async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined)
    render(<CommentComposer author={author} onSubmit={onSubmit} />)

    const textarea = screen.getByRole('textbox')
    await userEvent.type(textarea, 'Boden war noch feucht.')
    await userEvent.click(screen.getByRole('button', { name: 'Kommentieren' }))

    expect(onSubmit).toHaveBeenCalledWith('Boden war noch feucht.')
    expect(textarea).toHaveValue('')
  })

  it('does not clear the field when the submit rejects', async () => {
    const onSubmit = vi.fn().mockRejectedValue(new Error('network error'))
    render(<CommentComposer author={author} onSubmit={onSubmit} />)

    const textarea = screen.getByRole('textbox')
    await userEvent.type(textarea, 'Gießen verschoben.')
    await userEvent.click(screen.getByRole('button', { name: 'Kommentieren' }))

    expect(textarea).toHaveValue('Gießen verschoben.')
  })

  it('shows the character counter only past 80% of maxLength', async () => {
    render(<CommentComposer author={author} onSubmit={vi.fn()} maxLength={20} />)
    const textarea = screen.getByRole('textbox')

    await userEvent.type(textarea, '1234567890123456')
    expect(screen.queryByText(/Zeichen/)).not.toBeInTheDocument()

    await userEvent.type(textarea, '7')
    expect(screen.getByText('Noch 3 Zeichen')).toBeInTheDocument()
  })

  it('disables the textarea and shows a spinner while submitting', () => {
    render(<CommentComposer author={author} onSubmit={vi.fn()} isSubmitting />)
    expect(screen.getByRole('textbox')).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Kommentieren' })).toBeDisabled()
  })
})
