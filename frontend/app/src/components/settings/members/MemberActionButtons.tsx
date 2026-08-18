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
}: MemberActionButtonsProps) => (
  <>
    <Button
      type="button"
      variant="outline"
      onClick={onCancel}
      disabled={saving}
      className="w-full sm:w-auto"
    >
      Abbrechen
    </Button>
    <Button
      type="button"
      onClick={onSave}
      disabled={saving || phoneNumberInvalid}
      className="w-full sm:w-auto"
    >
      {saving && <Spinner className="size-4" />}
      Speichern
    </Button>
  </>
)

export default MemberActionButtons
