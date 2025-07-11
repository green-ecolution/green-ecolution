import { Link, LinkProps } from '@tanstack/react-router'
import { MoveRight } from 'lucide-react'
import React from 'react'

interface ButtonLinkProps {
  label: string
  link: LinkProps
  color?: 'green' | 'grey'
  asButton?: boolean
  iconClassName?: string
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>
}

const ButtonLink: React.FC<ButtonLinkProps> = ({
  label,
  icon: Icon = MoveRight,
  link,
  color = 'green',
  iconClassName = '',
}) => {
  const colorClasses = {
    green: 'bg-green-dark hover:bg-green-light text-white',
    grey: 'border border-dark-600 text-dark-600 hover:border-dark hover:text-dark',
  }

  return (
    <Link
      {...link}
      className={`w-fit whitespace-nowrap px-5 py-2 group flex gap-x-3 rounded-xl items-center transition-all ease-in-out duration-300 ${colorClasses[color]}`}
    >
      {label}
      {Icon && <Icon className={iconClassName} />}
    </Link>
  )
}

export default ButtonLink
