import { Link, LinkProps } from '@tanstack/react-router'
import { ReactNode } from 'react'

interface NavLinkProps extends LinkProps {
  label: string
  icon: ReactNode
  isExternalLink?: boolean
  navIsOpen?: boolean
  closeSidebar: () => void
}

const NavLink = (props: NavLinkProps) => {
  const {
    label,
    icon,
    isExternalLink = false,
    navIsOpen = false,
    closeSidebar,
    ...linkProps
  } = props

  return (
    <li className="relative">
      <Link
        target={isExternalLink ? '_blank' : '_self'}
        activeOptions={{ exact: false }}
        activeProps={{
          'aria-current': 'page',
          className: 'border-green-dark',
          style: { backgroundColor: 'color-mix(in oklch, var(--green-dark) 30%, transparent)' },
        }}
        inactiveProps={{
          className: 'border-transparent',
        }}
        onClick={closeSidebar}
        className="text-light border text-base block p-3.5 rounded-2xl transition-all ease-in-out duration-300 hover:bg-green-light/20 hover:text-green-light-200"
        {...linkProps}
      >
        {icon}
        <span
          className={`font-lato font-semibold tracking-[0.1] transition-all ease-in-out duration-300 absolute left-14 top-3
            ${navIsOpen ? 'lg:opacity-100 lg:block' : 'lg:opacity-0 lg:hidden'}`}
        >
          {label}
        </span>
      </Link>
    </li>
  )
}

export default NavLink
