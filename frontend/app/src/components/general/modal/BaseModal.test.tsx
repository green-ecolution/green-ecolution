import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { BaseModal } from './BaseModal'

describe('BaseModal', () => {
  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    title: 'Test Modal',
    children: <div>Modal Content</div>,
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    document.body.style.overflow = ''
  })

  describe('Rendering', () => {
    it('renders modal when isOpen is true', () => {
      render(<BaseModal {...defaultProps} />)

      expect(screen.getByRole('dialog')).toBeInTheDocument()
      expect(screen.getByText('Test Modal')).toBeInTheDocument()
      expect(screen.getByText('Modal Content')).toBeInTheDocument()
    })

    it('does not render when isOpen is false', () => {
      render(<BaseModal {...defaultProps} isOpen={false} />)

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })

    it('renders description when provided', () => {
      render(<BaseModal {...defaultProps} description="Test description" />)

      expect(screen.getByText('Test description')).toBeInTheDocument()
    })

    it('does not render description when not provided', () => {
      render(<BaseModal {...defaultProps} />)

      expect(screen.queryByText('Test description')).not.toBeInTheDocument()
    })

    it('renders close button', () => {
      render(<BaseModal {...defaultProps} />)

      expect(screen.getByRole('button', { name: /dialog schließen/i })).toBeInTheDocument()
    })
  })

  describe('Portal Rendering', () => {
    it('renders modal in document.body via portal', () => {
      const { baseElement } = render(
        <div id="app-root">
          <BaseModal {...defaultProps} />
        </div>,
      )

      const dialog = baseElement.querySelector('[role="dialog"]')
      expect(dialog?.parentElement).toBe(document.body)
    })
  })

  describe('Accessibility', () => {
    it('has aria-modal attribute', () => {
      render(<BaseModal {...defaultProps} />)

      expect(screen.getByRole('dialog')).toHaveAttribute('aria-modal', 'true')
    })

    it('has aria-labelledby pointing to title', () => {
      render(<BaseModal {...defaultProps} />)

      const dialog = screen.getByRole('dialog')
      const labelledBy = dialog.getAttribute('aria-labelledby')
      expect(labelledBy).toBeTruthy()

      const title = document.getElementById(labelledBy!)
      expect(title).toHaveTextContent('Test Modal')
    })

    it('has aria-describedby when description is provided', () => {
      render(<BaseModal {...defaultProps} description="Test description" />)

      const dialog = screen.getByRole('dialog')
      const describedBy = dialog.getAttribute('aria-describedby')
      expect(describedBy).toBeTruthy()

      const description = document.getElementById(describedBy!)
      expect(description).toHaveTextContent('Test description')
    })

    it('does not have aria-describedby when description is not provided', () => {
      render(<BaseModal {...defaultProps} />)

      const dialog = screen.getByRole('dialog')
      expect(dialog.getAttribute('aria-describedby')).toBeNull()
    })

    it('has aria-hidden on backdrop', () => {
      render(<BaseModal {...defaultProps} />)

      const backdrop = document.querySelector('[aria-hidden="true"]')
      expect(backdrop).toBeInTheDocument()
    })
  })

  describe('Close Behavior', () => {
    it('calls onClose when close button is clicked', async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()

      render(<BaseModal {...defaultProps} onClose={onClose} />)

      await user.click(screen.getByRole('button', { name: /dialog schließen/i }))

      expect(onClose).toHaveBeenCalledTimes(1)
    })

    it('calls onClose when backdrop is clicked', async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()

      render(<BaseModal {...defaultProps} onClose={onClose} />)

      const backdrop = document.querySelector('[aria-hidden="true"]')
      await user.click(backdrop!)

      expect(onClose).toHaveBeenCalledTimes(1)
    })

    it('does not call onClose when backdrop click is disabled', async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()

      render(<BaseModal {...defaultProps} onClose={onClose} closeOnBackdropClick={false} />)

      const backdrop = document.querySelector('[aria-hidden="true"]')
      await user.click(backdrop!)

      expect(onClose).not.toHaveBeenCalled()
    })

    it('calls onClose when Escape key is pressed', async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()

      render(<BaseModal {...defaultProps} onClose={onClose} />)

      await user.keyboard('{Escape}')

      expect(onClose).toHaveBeenCalledTimes(1)
    })

    it('does not call onClose when Escape is disabled', async () => {
      const user = userEvent.setup()
      const onClose = vi.fn()

      render(<BaseModal {...defaultProps} onClose={onClose} closeOnEscape={false} />)

      await user.keyboard('{Escape}')

      expect(onClose).not.toHaveBeenCalled()
    })
  })

  describe('Body Scroll Lock', () => {
    it('sets body overflow to hidden when modal opens', () => {
      render(<BaseModal {...defaultProps} />)

      expect(document.body.style.overflow).toBe('hidden')
    })

    it('restores body overflow when modal closes', async () => {
      const { rerender } = render(<BaseModal {...defaultProps} />)

      expect(document.body.style.overflow).toBe('hidden')

      rerender(<BaseModal {...defaultProps} isOpen={false} />)

      await waitFor(() => {
        expect(document.body.style.overflow).toBe('')
      })
    })
  })

  describe('Size Variants', () => {
    it('applies small size class', () => {
      render(<BaseModal {...defaultProps} size="sm" />)

      expect(screen.getByRole('dialog')).toHaveClass('max-w-sm')
    })

    it('applies medium size class by default', () => {
      render(<BaseModal {...defaultProps} />)

      expect(screen.getByRole('dialog')).toHaveClass('max-w-[30rem]')
    })

    it('applies large size class', () => {
      render(<BaseModal {...defaultProps} size="lg" />)

      expect(screen.getByRole('dialog')).toHaveClass('max-w-2xl')
    })
  })

  describe('Custom className', () => {
    it('applies custom className to modal', () => {
      render(<BaseModal {...defaultProps} className="custom-class" />)

      expect(screen.getByRole('dialog')).toHaveClass('custom-class')
    })
  })
})
