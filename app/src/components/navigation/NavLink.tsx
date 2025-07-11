import { Link } from '@tanstack/react-router'
import React, { ReactNode } from 'react'

interface NavLink {
  label: string
  url: string
  icon: ReactNode
  isExternalLink?: boolean
  navIsOpen?: boolean
  closeSidebar: () => void
}

const NavLink: React.FC<NavLink> = ({
  label,
  url,
  icon,
  isExternalLink = false,
  navIsOpen = false,
  closeSidebar,
}) => {
  return (
    <li className="relative">
      <Link
        to={url}
        target={isExternalLink ? '_blank' : '_self'}
        activeOptions={{ exact: false }}
        activeProps={{
          'aria-current': true,
        }}
        onClick={closeSidebar}
        className="text-light border border-transparent text-base block p-3.5 rounded-2xl transition-all ease-in-out duration-300 hover:bg-green-light/20 hover:text-green-light-200 
            aria-[current]:border-green-dark aria-[current]:bg-green-dark/30"
      >
        {icon}
        <span
          className={`font-lato font-semibold tracking-[0.1] transition-all ease-in-out duration-300 absolute left-14 top-3
            ${navIsOpen ? 'lg:opacity-full lg:block' : 'lg:opacity-0 lg:hidden'}`}
        >
          {label}
        </span>
      </Link>
    </li>
  )
}

export default NavLink
