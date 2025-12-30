import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import Select from './Select'

const defaultOptions = [
  { value: 'option1', label: 'Option 1' },
  { value: 'option2', label: 'Option 2' },
  { value: 'option3', label: 'Option 3' },
]

describe('Select', () => {
  describe('Rendering', () => {
    it('renders label correctly', () => {
      render(<Select label="Test Label" options={defaultOptions} />)
      expect(screen.getByText('Test Label')).toBeInTheDocument()
    })

    it('renders required indicator when required', () => {
      render(<Select label="Test Label" options={defaultOptions} required />)
      expect(screen.getByText('*')).toBeInTheDocument()
    })

    it('renders description when provided', () => {
      render(
        <Select label="Test Label" options={defaultOptions} description="This is a description" />,
      )
      expect(screen.getByText('This is a description')).toBeInTheDocument()
    })

    it('renders all options', () => {
      render(<Select label="Test Label" options={defaultOptions} />)
      expect(screen.getByRole('option', { name: 'Option 1' })).toBeInTheDocument()
      expect(screen.getByRole('option', { name: 'Option 2' })).toBeInTheDocument()
      expect(screen.getByRole('option', { name: 'Option 3' })).toBeInTheDocument()
    })

    it('renders placeholder when provided', () => {
      render(<Select label="Test Label" options={defaultOptions} placeholder="Select an option" />)
      expect(screen.getByRole('option', { name: 'Select an option' })).toBeInTheDocument()
    })

    it('renders error message when provided', () => {
      render(<Select label="Test Label" options={defaultOptions} error="This is an error" />)
      expect(screen.getByText('This is an error')).toBeInTheDocument()
    })

    it('renders chevron icon', () => {
      render(<Select label="Test Label" options={defaultOptions} />)
      const figure = document.querySelector('figure')
      expect(figure).toBeInTheDocument()
      expect(figure).toHaveAttribute('aria-hidden', 'true')
    })
  })

  describe('Interaction', () => {
    it('allows selecting an option', async () => {
      const user = userEvent.setup()
      render(<Select label="Test Label" options={defaultOptions} />)

      const select = screen.getByRole('combobox')
      await user.selectOptions(select, 'option2')

      expect(select).toHaveValue('option2')
    })

    it('calls onChange when option is selected', async () => {
      const user = userEvent.setup()
      const handleChange = vi.fn()
      render(<Select label="Test Label" options={defaultOptions} onChange={handleChange} />)

      const select = screen.getByRole('combobox')
      await user.selectOptions(select, 'option1')

      expect(handleChange).toHaveBeenCalled()
    })

    it('allows clicking on icon area to interact with select', async () => {
      const user = userEvent.setup()
      const handleChange = vi.fn()
      render(<Select label="Test Label" options={defaultOptions} onChange={handleChange} />)

      const figure = document.querySelector('figure')
      expect(figure).toHaveClass('pointer-events-none')

      const select = screen.getByRole('combobox')
      await user.selectOptions(select, 'option1')

      expect(handleChange).toHaveBeenCalled()
    })
  })

  describe('Accessibility', () => {
    it('associates label with select via name prop', () => {
      render(<Select label="Test Label" name="test-select" options={defaultOptions} />)

      const select = screen.getByRole('combobox')
      expect(select).toHaveAttribute('id', 'test-select')
    })

    it('marks icon as aria-hidden', () => {
      render(<Select label="Test Label" options={defaultOptions} />)

      const figure = document.querySelector('figure')
      expect(figure).toHaveAttribute('aria-hidden', 'true')
    })
  })
})
