import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import CreateOrganizationDialog from './CreateOrganizationDialog'

describe('CreateOrganizationDialog', () => {
  it('submits the trimmed name on Enter', async () => {
    const onSubmit = vi.fn()
    render(
      <CreateOrganizationDialog
        open
        parentName="Stadt Flensburg"
        saving={false}
        nameError={null}
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
      />,
    )
    await userEvent.type(
      screen.getByLabelText('Name der Organisation'),
      '  Grünflächenamt  {Enter}',
    )
    expect(onSubmit).toHaveBeenCalledWith('Grünflächenamt')
  })

  it('associates the name error with the input', () => {
    render(
      <CreateOrganizationDialog
        open
        parentName="Stadt Flensburg"
        saving={false}
        nameError="Eine Organisation mit diesem Namen existiert bereits."
        onOpenChange={vi.fn()}
        onSubmit={vi.fn()}
      />,
    )
    const input = screen.getByLabelText('Name der Organisation')
    expect(input).toHaveAccessibleDescription(
      'Eine Organisation mit diesem Namen existiert bereits.',
    )
    expect(input).toHaveAttribute('aria-invalid', 'true')
  })

  it('does nothing on Enter when the field is only whitespace', async () => {
    const onSubmit = vi.fn()
    render(
      <CreateOrganizationDialog
        open
        parentName="Stadt Flensburg"
        saving={false}
        nameError={null}
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
      />,
    )
    await userEvent.type(screen.getByLabelText('Name der Organisation'), '   {Enter}')
    expect(onSubmit).not.toHaveBeenCalled()
  })
})
