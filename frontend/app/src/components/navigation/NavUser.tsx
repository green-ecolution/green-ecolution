import { ChevronsUpDown, LogOut, UserRound } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import {
  Avatar,
  AvatarFallback,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { navItemClasses } from './navItemStyles'

interface NavUserProps {
  firstName: string
  lastName: string
  email: string
  collapsed: boolean
  closeSidebar: () => void
}

// Menu entries mirror the sidebar nav rows (see navItemStyles); focus: replaces
// hover: because Radix drives highlighting via focus.
const menuItemClasses =
  'cursor-pointer gap-x-3 rounded-xl px-3 py-2.5 font-lato font-semibold tracking-[0.1] text-light transition-all duration-300 ease-in-out focus:bg-green-light/20 focus:text-green-light-200 [&_svg]:size-5'

const NavUser = ({ firstName, lastName, email, collapsed, closeSidebar }: NavUserProps) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const initials = `${firstName.charAt(0)}${lastName.charAt(0)}`
  const fullName = `${firstName} ${lastName}`.trim()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          aria-label="Benutzermenü öffnen"
          className={`${navItemClasses} w-full border-transparent ${collapsed ? 'px-3 lg:justify-center lg:px-2' : 'px-2'}`}
        >
          <Avatar size="sm">
            <AvatarFallback variant="user">{initials}</AvatarFallback>
          </Avatar>
          <span
            className={`min-w-0 flex-1 text-left leading-tight ${collapsed ? 'lg:hidden' : ''}`}
          >
            <span className="block truncate font-lato font-semibold tracking-[0.1]">
              {fullName}
            </span>
            <span className="block truncate text-xs text-light/70">{email}</span>
          </span>
          <ChevronsUpDown className={`size-4 shrink-0 ${collapsed ? 'lg:hidden' : ''}`} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side={isLargeScreen ? 'right' : 'top'}
        align="end"
        sideOffset={8}
        className="w-72 rounded-lg border-dark-400/30 bg-dark px-2 pb-2 pt-5 text-sm text-white shadow-cards"
      >
        <p className="mx-3 pb-4">
          Angemeldet als:
          <strong className="block truncate">{email}</strong>
        </p>
        <DropdownMenuSeparator className="mx-3 bg-dark-800" />
        <div className="pt-2">
          <DropdownMenuItem asChild className={menuItemClasses}>
            <Link to="/profile" onClick={closeSidebar}>
              <UserRound />
              Ihr Profil
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild className={menuItemClasses}>
            <Link to="/logout" preload={false} onClick={closeSidebar}>
              <LogOut />
              Abmelden
            </Link>
          </DropdownMenuItem>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export default NavUser
