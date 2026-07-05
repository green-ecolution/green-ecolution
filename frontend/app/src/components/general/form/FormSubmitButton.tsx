import { Button } from '@green-ecolution/ui'
import { MoveRight } from 'lucide-react'

interface FormSubmitButtonProps {
  disabled: boolean
  className?: string
}

/** Passing `className` replaces the default layout classes instead of merging with them. */
const FormSubmitButton = ({
  disabled,
  className = 'mt-10 lg:col-span-full lg:w-fit',
}: FormSubmitButtonProps) => (
  <Button type="submit" className={className} disabled={disabled}>
    Speichern
    <MoveRight className="icon-arrow-animate" />
  </Button>
)

export default FormSubmitButton
