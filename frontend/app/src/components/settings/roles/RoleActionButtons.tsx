import { Button, Spinner } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'

interface RoleActionButtonsProps {
  isNew: boolean
  saving: boolean
  nameEmpty: boolean
  onSave: () => void
  onCancel: () => void
}

const RoleActionButtons = ({
  isNew,
  saving,
  nameEmpty,
  onSave,
  onCancel,
}: RoleActionButtonsProps) => {
  const { t } = useTranslation(['settings', 'common'])

  return (
    <>
      <Button
        type="button"
        variant="outline"
        onClick={onCancel}
        disabled={saving}
        className="w-full sm:w-auto"
      >
        {t('common:actions.cancel')}
      </Button>
      <Button
        type="button"
        onClick={onSave}
        disabled={saving || nameEmpty}
        className="w-full sm:w-auto"
      >
        {saving && <Spinner className="size-4" />}
        {isNew ? t('roles.createButtonLabel') : t('common:actions.save')}
      </Button>
    </>
  )
}

export default RoleActionButtons
