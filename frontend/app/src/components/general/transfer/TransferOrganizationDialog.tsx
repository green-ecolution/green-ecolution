import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowRightLeft } from 'lucide-react'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import {
  Button,
  Combobox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogIcon,
  DialogTitle,
  Label,
} from '@green-ecolution/ui'

type TargetOrganization = Pick<OrganizationResponse, 'id' | 'name'>

interface TransferOrganizationDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  /** Target candidates, see `useTransferTargets`. */
  organizations: TargetOrganization[]
  onConfirm: (organizationId: string) => void
  pending?: boolean
}

interface TransferFormProps {
  organizations: TargetOrganization[]
  pending: boolean
  onCancel: () => void
  onConfirm: (organizationId: string) => void
}

// Lives inside DialogContent, which Radix unmounts when closed, so every
// opening starts without a choice even when the caller closed the dialog.
const TransferForm = ({ organizations, pending, onCancel, onConfirm }: TransferFormProps) => {
  const { t } = useTranslation('common')
  const [organizationId, setOrganizationId] = useState('')

  return (
    <>
      <div className="flex flex-col gap-y-2">
        <Label htmlFor="transfer-organization">{t('transfer.organizationLabel')}</Label>
        <Combobox
          id="transfer-organization"
          options={organizations.map((org) => ({ value: org.id, label: org.name }))}
          value={organizationId}
          onChange={setOrganizationId}
          placeholder={t('transfer.organizationPlaceholder')}
          searchPlaceholder={t('transfer.organizationSearchPlaceholder')}
          emptyText={t('transfer.organizationEmpty')}
          disabled={pending}
        />
      </div>

      <DialogFooter>
        <Button type="button" variant="outline" disabled={pending} onClick={onCancel}>
          {t('actions.cancel')}
        </Button>
        <Button
          type="button"
          disabled={pending || organizationId === ''}
          onClick={() => onConfirm(organizationId)}
        >
          {pending ? t('transfer.pending') : t('transfer.confirm')}
        </Button>
      </DialogFooter>
    </>
  )
}

const TransferOrganizationDialog = ({
  open,
  onOpenChange,
  title,
  description,
  organizations,
  onConfirm,
  pending = false,
}: TransferOrganizationDialogProps) => {
  const handleOpenChange = (next: boolean) => {
    if (!pending) onOpenChange(next)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogIcon variant="info">
            <ArrowRightLeft />
          </DialogIcon>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <TransferForm
          organizations={organizations}
          pending={pending}
          onCancel={() => handleOpenChange(false)}
          onConfirm={onConfirm}
        />
      </DialogContent>
    </Dialog>
  )
}

export default TransferOrganizationDialog
