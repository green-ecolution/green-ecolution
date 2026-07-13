import { AlignJustifyIcon, ChevronDown } from 'lucide-react'
import { useState, useCallback } from 'react'
import Navigation from './Navigation'
import Breadcrumb from './Breadcrumb'
import NavUserMenu from '../navigation/NavUserMenu'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { Avatar, AvatarFallback, Button } from '@green-ecolution/ui'

function Header() {
  const [open, setOpen] = useState(false)
  const isStartPage = location.pathname === '/'
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const collapsed = useSidebarCollapsed()
  const { isAuthenticated } = useAuthSession()
  const { firstName, lastName, email } = useCurrentUser()

  const closeSidebar = useCallback(() => {
    setOpen(false)
    if (!isLargeScreen) {
      document.body.classList.remove('overflow-y-hidden')
    }
  }, [isLargeScreen])

  const toggleSidebar = useCallback(() => {
    setOpen((prev) => {
      const next = !prev
      if (!isLargeScreen) {
        document.body.classList.toggle('overflow-y-hidden', next)
      }
      return next
    })
  }, [isLargeScreen])

  // z-50: the fullscreen mobile nav overlay lives inside this stacking
  // context, so it must sit on the overlay layer or page content (e.g.
  // map controls) paints above it
  return (
    <header
      className={`relative z-50 bg-white transition-[padding] ease-in-out duration-300 ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'}`}
    >
      {/* min-h keeps the pre-NavUser header height (40px avatar + py-4 + border);
          the map height calc (100dvh - 4.563rem) depends on it */}
      <div className="container min-h-[4.563rem] text-sm border-b border-dark-50 py-4 flex justify-between items-center">
        {!isLargeScreen && (
          <Button
            id="main-navigation-toggle"
            variant="ghost"
            size="icon"
            aria-expanded={open}
            aria-controls="main-navigation"
            aria-haspopup="menu"
            aria-label="Hauptnavigation öffnen"
            className="size-8 rounded-full bg-dark hover:bg-dark-600"
            onClick={toggleSidebar}
          >
            <AlignJustifyIcon className="!size-5 text-light" />
          </Button>
        )}
        {!isStartPage && <Breadcrumb />}
        {!isLargeScreen && isAuthenticated && (
          <div className="ml-auto">
            <NavUserMenu email={email} side="bottom" onNavigate={closeSidebar}>
              <button
                type="button"
                aria-label="Benutzermenü öffnen"
                className="group flex cursor-pointer items-center gap-x-1"
              >
                <Avatar>
                  <AvatarFallback variant="user">
                    {`${firstName.charAt(0)}${lastName.charAt(0)}`}
                  </AvatarFallback>
                </Avatar>
                <ChevronDown className="size-5 text-dark transition-transform duration-300 ease-in-out group-data-[state=open]:rotate-180" />
              </button>
            </NavUserMenu>
          </div>
        )}
      </div>

      <Navigation isOpen={open} closeSidebar={closeSidebar} />
    </header>
  )
}

export default Header
