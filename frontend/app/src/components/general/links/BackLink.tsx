import { Link, LinkProps } from '@tanstack/react-router'
import { MoveLeft } from 'lucide-react'
import React from 'react'

interface BackLinkProps {
  label: string
  link: LinkProps
}

const BackLink: React.FC<BackLinkProps> = ({ label, link }) => (
  <Link
    className="group flex items-center gap-x-2 text-dark-600 font-medium text-sm mb-4"
    {...link}
  >
    <MoveLeft className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:-translate-x-1" />
    {label}
  </Link>
)

export default BackLink
