import { Link, LinkProps } from '@tanstack/react-router'
import { ReactNode } from 'react'
import { navItemClasses } from './navItemStyles'

interface NavLinkProps extends LinkProps {
  label: string
  icon: ReactNode
  isExternalLink?: boolean
  collapsed?: boolean
  closeSidebar: () => void
}

const NavLink = (props: NavLinkProps) => {
  const {
    label,
    icon,
    isExternalLink = false,
    collapsed = false,
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
          className: 'border-green-dark bg-green-dark/30',
        }}
        inactiveProps={{
          className: 'border-transparent',
        }}
        onClick={closeSidebar}
        title={label}
        aria-label={label}
        className={`${navItemClasses} px-3 ${collapsed ? 'lg:justify-center lg:px-2' : ''}`}
        {...linkProps}
      >
        <span className="shrink-0">{icon}</span>
        <span className={`font-lato font-semibold tracking-[0.1] ${collapsed ? 'lg:hidden' : ''}`}>
          {label}
        </span>
      </Link>
    </li>
  )
}

export default NavLink
