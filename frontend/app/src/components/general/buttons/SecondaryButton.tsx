import { X } from 'lucide-react'
import React from 'react'

interface SecondaryButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  label: string
  type?: 'submit' | 'reset' | 'button'
}

const SecondaryButton: React.FC<SecondaryButtonProps> = ({ label, type = 'button', ...props }) => (
  <button
    type={type}
    {...props}
    className={`border border-green-dark text-green-dark px-5 py-2 group flex gap-x-3 rounded-xl items-center transition-all ease-in-out duration-300 hover:border-green-light hover:text-green-light ${props.className}`}
  >
    <span className="font-medium text-base">{label}</span>
    <X />
  </button>
)

export default SecondaryButton
