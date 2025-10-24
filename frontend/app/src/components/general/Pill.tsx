import { StatusColor } from '@/hooks/details/useDetailsForWateringPlanStatus'
import React from 'react'

interface PillProps {
  label: string
  theme?: StatusColor
}

const Pill: React.FC<PillProps> = ({ label, theme = 'dark-400' }) => {
  const themeClasses = {
    red: 'border-red text-red',
    yellow: 'border-yellow text-yellow',
    'dark-400': 'border-dark-600 text-dark-600',
    'dark-600': 'border-dark-600 text-dark-600',
    'green-dark': 'border-green-dark text-green-dark',
    'green-light': 'text-green-dark border-green-light bg-green-light-200',
  }
  return (
    <span
      className={`text-sm font-medium w-fit border rounded-full px-4 py-1 ${themeClasses[theme]}`}
    >
      {label}
    </span>
  )
}

export default Pill
