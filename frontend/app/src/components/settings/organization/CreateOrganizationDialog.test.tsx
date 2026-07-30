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
