import { useTranslation } from 'react-i18next'
import { Button, Spinner } from '@green-ecolution/ui'

interface MemberActionButtonsProps {
  saving: boolean
  phoneNumberInvalid: boolean
  onSave: () => void
  onCancel: () => void
}

const MemberActionButtons = ({
  saving,
  phoneNumberInvalid,
  onSave,
  onCancel,
}: MemberActionButtonsProps) => {
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
        disabled={saving || phoneNumberInvalid}
        className="w-full sm:w-auto"
      >
        {saving && <Spinner className="size-4" />}
        {t('actions.save')}
      </Button>
    </>
  )
}

export default MemberActionButtons
