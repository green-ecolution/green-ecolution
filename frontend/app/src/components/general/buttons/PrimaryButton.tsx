import { MoveRight } from 'lucide-react'
import React from 'react'

interface PrimaryButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  label: string
  isDanger?: boolean
  type?: 'submit' | 'reset' | 'button'
}

const PrimaryButton: React.FC<PrimaryButtonProps> = ({
  label,
  type = 'button',
  isDanger = false,
  ...props
}) => (
  <button
    type={type}
    {...props}
    className={`${isDanger ? 'bg-red' : 'bg-green-dark hover:bg-green-light'} text-white w-fit px-5 py-2 group flex gap-x-3 rounded-xl items-center transition-all ease-in-out duration-300 disabled:bg-dark-400 ${props.className}`}
  >
    <span className="font-medium text-base">{label}</span>
    <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2 group-disabled:translate-x-0" />
  </button>
)

export default PrimaryButton
