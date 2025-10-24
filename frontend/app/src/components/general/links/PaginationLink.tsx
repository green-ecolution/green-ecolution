import { Link, LinkProps } from '@tanstack/react-router'
import React from 'react'

interface PaginationLinkProps {
  color?: 'green-light' | 'grey' | 'green-dark'
  link: LinkProps
  label?: number
  disabled?: boolean
  icon?: React.ComponentType<React.SVGProps<SVGSVGElement>>
}

const PaginationLink: React.FC<PaginationLinkProps> = ({
  color = 'grey',
  link,
  label,
  icon: Icon,
  disabled = false,
}) => {
  const colorClasses = {
    'green-light': 'bg-green-light/20 border border-green-light text-green-dark',
    'green-dark':
      'border border-green-dark text-green-dark aria-disabled:border-dark-200 aria-disabled:text-dark-400',
    grey: 'bg-dark-50 text-dark-600 hover:bg-dark-100',
  }

  return (
    <Link
      {...link}
      disabled={disabled}
      className={`flex items-center justify-center size-8 rounded-full font-semibold transition-color ease-in-out duration-300 ${colorClasses[color]}`}
    >
      {label && <span>{label}</span>}
      {Icon && <Icon className="size-4" />}
    </Link>
  )
}

export default PaginationLink
