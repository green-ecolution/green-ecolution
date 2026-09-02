import { useTranslation } from 'react-i18next'

interface FormErrorProps {
  show: boolean
  error?: string
}

// The resolved message already names the cause, so repeating a generic
// sentence above it only buried the useful line.
const FormError = ({ error, show }: FormErrorProps) => {
  const { t } = useTranslation('common')

  return (
    <div className={`text-destructive font-semibold text-sm mt-10 ${show ? '' : 'hidden'}`}>
      <p>{error ?? t('form.error.fallback')}</p>
    </div>
  )
}

export default FormError
