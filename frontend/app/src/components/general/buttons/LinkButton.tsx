import { MoveRight } from 'lucide-react'
import React from 'react'

interface LinkButtonProps {
  onClick: () => void
  label: string
  color?: 'green' | 'red'
}

const LinkAsButton: React.FC<LinkButtonProps> = ({ label, onClick, color = 'green' }) => (
  <button
    type="button"
    onClick={onClick}
    className={`group flex items-center gap-x-2 font-medium text-base ${color === 'green' ? 'text-green-dark' : 'text-red'}`}
  >
    {label}
    <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
  </button>
)

export default LinkAsButton
