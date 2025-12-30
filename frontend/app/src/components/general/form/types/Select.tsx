import { ChevronDown } from 'lucide-react'
import { Ref, useId } from 'react'

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label: string
  placeholder?: string
  options: { value: string; label: string }[]
  error?: string
  description?: string
  ref?: Ref<HTMLSelectElement>
}

const Select = ({ ref, ...props }: SelectProps) => {
  const generatedId = useId()
  const selectId = props.id ?? props.name ?? generatedId

  return (
    <div>
      <label htmlFor={selectId} className="block font-semibold text-dark-800 mb-2.5">
        {props.label} {props.required ? <span className="text-red">*</span> : null}
      </label>
      {props.description && (
        <p className="-mt-2 text-sm text-dark-600 mb-2.5">{props.description}</p>
      )}
      <div className="relative">
        <select
          ref={ref}
          id={selectId}
          multiple={props.multiple}
          className={`w-full text-dark-800 border border-green-light rounded-lg bg-white px-4 py-3 focus:outline-green-dark ${props.multiple ? '' : 'appearance-none pr-12'}`}
          {...props}
        >
          {props.placeholder && (
            <option value="" disabled>
              {props.placeholder}
            </option>
          )}
          {props.options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {!props.multiple && (
          <figure
            aria-hidden="true"
            className="pointer-events-none absolute right-4 top-[1.125rem]"
          >
            <ChevronDown className="w-4 h-4 text-dark-800" />
          </figure>
        )}
      </div>
      {props.error && (
        <span className="block text-red mt-2 font-semibold text-sm">{props.error}</span>
      )}
    </div>
  )
}

export default Select
