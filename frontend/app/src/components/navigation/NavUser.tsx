import { ChevronsUpDown } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@green-ecolution/ui'
import NavUserMenu from './NavUserMenu'
import { navItemClasses } from './navItemStyles'

interface NavUserProps {
  firstName: string
  lastName: string
  email: string
  avatarUrl?: string
  collapsed: boolean
  closeSidebar: () => void
}

const NavUser = ({
  firstName,
  lastName,
  email,
  avatarUrl,
  collapsed,
  closeSidebar,
}: NavUserProps) => {
  const initials = `${firstName.charAt(0)}${lastName.charAt(0)}`
  const fullName = `${firstName} ${lastName}`.trim()

  return (
    <NavUserMenu email={email} side="right" onNavigate={closeSidebar}>
      <button
        type="button"
        aria-label="Benutzermenü öffnen"
        className={`${navItemClasses} w-full border-transparent ${collapsed ? 'px-3 lg:justify-center lg:px-2' : 'px-2'}`}
      >
        <Avatar size="sm">
          {avatarUrl && <AvatarImage src={avatarUrl} alt="" />}
          <AvatarFallback variant="user">{initials}</AvatarFallback>
        </Avatar>
        <span className={`min-w-0 flex-1 text-left leading-tight ${collapsed ? 'lg:hidden' : ''}`}>
          <span className="block truncate font-lato font-semibold tracking-[0.1]">{fullName}</span>
          <span className="block truncate text-xs text-light/70">{email}</span>
        </span>
        <ChevronsUpDown className={`size-4 shrink-0 ${collapsed ? 'lg:hidden' : ''}`} />
      </button>
    </NavUserMenu>
  )
}

export default NavUser
