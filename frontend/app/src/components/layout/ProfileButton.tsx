import { ChevronDown, LogIn, LogOut, Settings, UserRound } from 'lucide-react'
import useStore from '@/store/store'
import { useState } from 'react'
import NavLink from '../navigation/NavLink'
import useOutsideClick from '@/hooks/useOutsideClick'
import { Avatar, AvatarFallback } from '@green-ecolution/ui'

function ProfileButton() {
  const [open, setOpen] = useState(false)
  const firstName = useStore((state) => state.firstName)
  const lastName = useStore((state) => state.lastName)
  const email = useStore((state) => state.email)
  const isAuthenticated = useStore((state) => state.isAuthenticated)

  const toggleOverlay = (state: boolean) => {
    setOpen(state)
  }

  const overlayRef = useOutsideClick<HTMLDivElement>(() => toggleOverlay(false))
  const userInitials = `${firstName.charAt(0)}${lastName.charAt(0)}`

  const linksLoggedIn = [
    <NavLink
      key="nav-your-profile"
      label="Ihr Profil"
      icon={<Settings className="w-5 h-5" />}
      to="/profile"
      navIsOpen={true}
      closeSidebar={() => toggleOverlay(false)}
    />,
    <NavLink
      key="nav-logout"
      label="Abmelden"
      icon={<LogOut className="w-5 h-5" />}
      to="/logout"
      preload={false}
      navIsOpen={true}
      closeSidebar={() => toggleOverlay(false)}
    />,
  ]

  const linksLoggedOut = [
    <NavLink
      key="nav-header-login"
      label="Anmelden"
      icon={<LogIn className="w-5 h-5" />}
      to="/login"
      preload={false}
      navIsOpen={true}
      closeSidebar={() => toggleOverlay(false)}
    />,
  ]

  const links = isAuthenticated ? linksLoggedIn : linksLoggedOut

  return (
    <div className="relative ml-auto" ref={overlayRef}>
      <button
        type="button"
        aria-label="Profilinformationen anzeigen"
        aria-expanded={open}
        aria-controls="profile-informations"
        aria-haspopup="grid"
        className="group flex items-center gap-x-1 cursor-pointer"
        onClick={() => toggleOverlay(!open)}
      >
        <Avatar>
          <AvatarFallback variant={isAuthenticated ? 'user' : 'guest'}>
            {isAuthenticated ? userInitials : <UserRound className="w-5 h-5 stroke-2" />}
          </AvatarFallback>
        </Avatar>
        <ChevronDown
          className={`w-5 h-5 text-dark transition-all ease-in-out duration-300 ${open ? 'rotate-180' : ''}`}
        />
      </button>

      <div
        id="profile-informations"
        className={`bg-dark shadow-cards w-72 z-50 text-sm text-white pt-5 px-2 right-0 rounded-lg absolute top-14 ${open ? 'block' : 'hidden'}`}
      >
        <p className="border-b border-dark-800 mx-3 pb-4">
          {isAuthenticated ? <span>Angemeldet als:</span> : <span>Nicht angemeldet</span>}
          <br />
          <strong className="block truncate">{email}</strong>
        </p>

        <ul className="py-2">{links.map((link) => link)}</ul>
      </div>
    </div>
  )
}

export default ProfileButton
