import { Ref } from 'react'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  placeholder?: string
  label: string
  hideLabel?: boolean
  error?: string
  small?: boolean
  ref?: Ref<HTMLInputElement>
}

const Input = ({ hideLabel, error, small, label, ref, ...props }: InputProps) => {
  return (
    <div>
      {label && (
        <label
          htmlFor={props.name}
          className={`block font-semibold text-dark-800 mb-2.5 ${hideLabel ? 'sr-only' : ''}`}
        >
          {label} {props.required ? <span className="text-red">*</span> : null}
        </label>
      )}

      <input
        ref={ref}
        id={props.name}
        className={`w-full text-dark-800 border border-green-light rounded-lg bg-white px-4 py-3 focus:outline-green-dark ${small ? 'max-w-32' : ''}`}
        {...props}
      />
      {error && <span className="block text-red mt-2 font-semibold text-sm">{error}</span>}
    </div>
  )
}

export default Input
