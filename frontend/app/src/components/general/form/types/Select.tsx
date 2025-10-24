import { ChevronDown } from 'lucide-react'
import { Ref } from 'react'

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label: string
  placeholder?: string
  options: { value: string; label: string }[]
  error?: string
  description?: string
  ref?: Ref<HTMLSelectElement>
}

const Select = ({ ref, ...props }: SelectProps) => {
  return (
    <div>
      <label htmlFor={props.name} className="block font-semibold text-dark-800 mb-2.5">
        {props.label} {props.required ? <span className="text-red">*</span> : null}
      </label>
      {props.description && (
        <p className="-mt-2 text-sm text-dark-600 mb-2.5">{props.description}</p>
      )}
      <div className="relative">
        <select
          ref={ref}
          id={props.name}
          multiple={props.multiple}
          className="w-full text-dark-800 border border-green-light rounded-lg bg-white px-4 py-3 focus:outline-green-dark"
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
        <figure aria-hidden="true" className="absolute right-4 top-[1.125rem]">
          <ChevronDown className="w-4 h-4 text-dark-800" />
        </figure>
      </div>
      {props.error && (
        <span className="block text-red mt-2 font-semibold text-sm">{props.error}</span>
      )}
    </div>
  )
}

export default Select
