import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { CommentSubject } from '@/api/queries'
import type { Permissions } from '@/lib/auth/permissions'
import { renderInRouter } from '../test/renderInRouter'
import { BIN_FULL_ID, wasteBinFixtures } from '../test/handlers'
import { wasteBinApi } from '../api/client'

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

// The core section is tested on its own; here only the subject it gets matters.
vi.mock('@/components/comments/CommentsSection', () => ({
  default: ({ subject }: { subject: CommentSubject }) => (
    <section aria-label="Kommentare">
      {subject.kind === 'target'
        ? `${subject.target.module}/${subject.target.kind}/${subject.target.id}`
        : subject.id}
    </section>
  ),
}))

const { default: WasteBinDetail } = await import('./WasteBinDetail')

const [fullBin, normalBin] = wasteBinFixtures

describe('WasteBinDetail', { timeout: 20_000 }, () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    permissions.mockReturnValue(new Set(['waste_bin:read']))
  })

  it('shows master data and fill level', async () => {
    renderInRouter(<WasteBinDetail bin={fullBin} />)

    const masterData = await screen.findByRole('region', { name: 'Stammdaten' })
    expect(within(masterData).getByText('1100 l')).toBeInTheDocument()
    expect(within(masterData).getByText('120 cm')).toBeInTheDocument()

    const fill = screen.getByRole('region', { name: 'Füllstand' })
    expect(within(fill).getByText('Voll · 85 %')).toBeInTheDocument()
    expect(within(fill).getByText('Letzte Messung')).toBeInTheDocument()
    expect(within(fill).getByText(/27\.10\.2026/)).toBeInTheDocument()
    expect(within(fill).getByText('Letzte Leerung')).toBeInTheDocument()
    expect(within(fill).getByText(/20\.10\.2026/)).toBeInTheDocument()
  })

  it('links the attached sensor', async () => {
    renderInRouter(<WasteBinDetail bin={fullBin} />)
    const link = await screen.findByRole('link', { name: /eui-a84041000181c2f1/ })
    expect(link).toHaveAttribute('href', '/sensors/eui-a84041000181c2f1')
  })

  it('says so when no sensor is attached and nothing was emptied yet', async () => {
    renderInRouter(<WasteBinDetail bin={normalBin} />)
    const fill = await screen.findByRole('region', { name: 'Füllstand' })
    expect(within(fill).getByText('Kein Sensor verbunden')).toBeInTheDocument()
    expect(within(fill).getByText('Noch nicht geleert')).toBeInTheDocument()
  })

  it('shows the comments of the bin target', async () => {
    renderInRouter(<WasteBinDetail bin={fullBin} />)
    expect(await screen.findByRole('region', { name: 'Kommentare' })).toHaveTextContent(
      `waste/bin/${BIN_FULL_ID}`,
    )
  })

  it('hides edit and delete without the rights', async () => {
    renderInRouter(<WasteBinDetail bin={fullBin} />)
    await screen.findByRole('region', { name: 'Stammdaten' })
    expect(screen.queryByRole('link', { name: /Behälter bearbeiten/ })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Weitere Aktionen' })).not.toBeInTheDocument()
  })

  it('offers editing with waste_bin:update', async () => {
    permissions.mockReturnValue(new Set(['waste_bin:update']))
    renderInRouter(<WasteBinDetail bin={fullBin} />)
    const link = await screen.findByRole('link', { name: /Behälter bearbeiten/ })
    expect(link).toHaveAttribute('href', `/waste-bins/${BIN_FULL_ID}/edit`)
  })

  it('deletes only after confirmation with waste_bin:delete', async () => {
    permissions.mockReturnValue(new Set(['waste_bin:delete']))
    const deleteBin = vi.spyOn(wasteBinApi, 'deleteWasteBin').mockResolvedValue()
    const user = userEvent.setup()
    renderInRouter(<WasteBinDetail bin={fullBin} />)

    await user.click(await screen.findByRole('button', { name: 'Weitere Aktionen' }))
    await user.click(await screen.findByRole('menuitem', { name: /Behälter löschen/ }))

    const dialog = await screen.findByRole('alertdialog')
    expect(within(dialog).getByText('Behälter B-0042 löschen?')).toBeInTheDocument()
    expect(deleteBin).not.toHaveBeenCalled()

    await user.click(within(dialog).getByRole('button', { name: /Löschen/ }))
    await waitFor(() => expect(deleteBin).toHaveBeenCalledWith({ wasteBinId: BIN_FULL_ID }))
  })

  it('keeps the bin when the confirmation is cancelled', async () => {
    permissions.mockReturnValue(new Set(['waste_bin:delete']))
    const deleteBin = vi.spyOn(wasteBinApi, 'deleteWasteBin').mockResolvedValue()
    const user = userEvent.setup()
    renderInRouter(<WasteBinDetail bin={fullBin} />)

    await user.click(await screen.findByRole('button', { name: 'Weitere Aktionen' }))
    await user.click(await screen.findByRole('menuitem', { name: /Behälter löschen/ }))
    await user.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: /Abbrechen/ }),
    )

    await waitFor(() => expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument())
    expect(deleteBin).not.toHaveBeenCalled()
  })
})
