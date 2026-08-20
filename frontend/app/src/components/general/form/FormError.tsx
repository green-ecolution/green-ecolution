interface FormErrorProps {
  show: boolean
  error?: string
}

const FALLBACK =
  'Es ist leider ein Problem aufgetreten. Bitte probiere es erneut oder wende dich an einen Systemadministrierenden.'

// The resolved message already names the cause, so repeating a generic
// sentence above it only buried the useful line.
const FormError = ({ error, show }: FormErrorProps) => {
  return (
    <div className={`text-destructive font-semibold text-sm mt-10 ${show ? '' : 'hidden'}`}>
      <p>{error ?? FALLBACK}</p>
    </div>
  )
}

export default FormError
