import { useTranslation } from 'react-i18next'
import { Button, Spinner } from '@green-ecolution/ui'

interface OrganizationActionButtonsProps {
  saving: boolean
  /** True while at least one of the two endpoints has something valid to send.
   *  The halves are judged separately: a half-filled address blocks the master
   *  data, not a settings edit. */
  canSave: boolean
  onSave: () => void
  onCancel: () => void
}

const OrganizationActionButtons = ({
  saving,
  canSave,
  onSave,
  onCancel,
}: OrganizationActionButtonsProps) => {
  const { t } = useTranslation('common')

  return (
    <>
      <Button
        type="button"
        variant="outline"
        onClick={onCancel}
        disabled={saving}
        className="w-full sm:w-auto"
      >
        {t('actions.cancel')}
      </Button>
      <Button
        type="button"
        onClick={onSave}
        disabled={saving || !canSave}
        className="w-full sm:w-auto"
      >
        {saving && <Spinner className="size-4" />}
        {t('actions.save')}
      </Button>
    </>
  )
}

export default OrganizationActionButtons
