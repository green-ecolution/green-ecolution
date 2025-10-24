import { Link, LinkProps } from '@tanstack/react-router'
import { MoveRight } from 'lucide-react'
import React from 'react'

interface GeneralLink {
  label: string
  link: LinkProps
  theme?: 'grey' | 'green'
}

const GeneralLink: React.FC<GeneralLink> = ({ label, link, theme = 'green' }) => {
  return (
    <Link
      {...link}
      className={`group flex items-center gap-x-2 font-medium text-base ${theme === 'green' ? 'text-green-dark' : 'text-dark-600'}`}
    >
      {label}
      <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
    </Link>
  )
}

export default GeneralLink
