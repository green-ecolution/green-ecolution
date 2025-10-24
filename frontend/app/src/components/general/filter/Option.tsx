import React from 'react'

interface OptionProps {
  name: string
  label: string
  value?: string
  children?: React.ReactNode
  checked: boolean
  onChange: (event: React.ChangeEvent<HTMLInputElement>) => void
}

const Option: React.FC<OptionProps> = ({ name, label, value, children, checked, onChange }) => {
  return (
    <label
      className={`cursor-pointer mr-2 mb-2 inline-flex items-center gap-x-2 border w-max pr-5 pl-3 py-2 rounded-full border-green-light transition-all ease-in-out duration-300 hover:border-green-dark focus-within:border-green-dark
        ${checked ? 'bg-green-light-200' : ''}`}
    >
      <input
        type="checkbox"
        name={name}
        checked={checked}
        value={value ?? label}
        onChange={onChange}
        className="opacity-0 w-0 h-0"
      />
      {children && <div>{children}</div>}
      <span>{label}</span>
    </label>
  )
}

export default Option
