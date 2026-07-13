import { ReactNode } from 'react'
import { LogOut, UserRound } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'

interface NavUserMenuProps {
  email: string
  side: 'right' | 'bottom'
  onNavigate: () => void
  children: ReactNode
}

// Menu entries mirror the sidebar nav rows (see navItemStyles); focus: replaces
// hover: because Radix drives highlighting via focus.
const menuItemClasses =
  'cursor-pointer gap-x-3 rounded-xl px-3 py-2.5 font-lato font-semibold tracking-[0.1] text-light transition-all duration-300 ease-in-out focus:bg-green-light/20 focus:text-green-light-200 [&_svg]:size-5'

const NavUserMenu = ({ email, side, onNavigate, children }: NavUserMenuProps) => (
  <DropdownMenu>
    <DropdownMenuTrigger asChild>{children}</DropdownMenuTrigger>
    <DropdownMenuContent
      side={side}
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
          <Link to="/profile" onClick={onNavigate}>
            <UserRound />
            Ihr Profil
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild className={menuItemClasses}>
          <Link to="/logout" preload={false} onClick={onNavigate}>
            <LogOut />
            Abmelden
          </Link>
        </DropdownMenuItem>
      </div>
    </DropdownMenuContent>
  </DropdownMenu>
)

export default NavUserMenu
