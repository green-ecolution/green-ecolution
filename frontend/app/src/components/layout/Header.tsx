import { AlignJustifyIcon, ChevronDown } from 'lucide-react'
import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import Navigation from './Navigation'
import Breadcrumb from './Breadcrumb'
import NavUserMenu from '../navigation/NavUserMenu'
import ContextHelpLink from '../handbook/ContextHelpLink'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { useCurrentUserAvatar } from '@/lib/auth/useCurrentUserAvatar'
import { Avatar, AvatarFallback, AvatarImage, Button } from '@green-ecolution/ui'
import SidebarToggle from '../navigation/SidebarToggle'
import useStore from '@/store/store'

function Header() {
  const [open, setOpen] = useState(false)
  const isStartPage = location.pathname === '/'
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const collapsed = useSidebarCollapsed()
  const { isAuthenticated } = useAuthSession()
  const { firstName, lastName, email } = useCurrentUser()
  const avatarUrl = useCurrentUserAvatar()
  const setSidebarCollapsed = useStore((s) => s.setSidebarCollapsed)
  const { t } = useTranslation('navigation')

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

  // The fullscreen mobile nav lives inside this stacking context. z-50 keeps it
  // above page content; while open it goes one layer higher, because portals on
  // z-50 (e.g. the map's bottom drawer) come later in the DOM and would win.
  // z-index transitions with the nav's slide so it only drops back to 50 once
  // the closing nav is off screen.
  return (
    <header
      className={`sticky top-0 pt-[env(safe-area-inset-top)] ${open ? 'z-[60]' : 'z-50'} bg-white transition-[padding,z-index] ease-in-out duration-300 motion-reduce:transition-none ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'}`}
    >
      {/* min-h keeps the pre-NavUser header height (40px avatar + py-4 + border) */}
      <div className="container min-h-[4.563rem] text-sm border-b border-dark-50 py-4 flex justify-start items-center">
        {isLargeScreen && (
          <div className="mr-4 flex items-center">
            <SidebarToggle collapsed={collapsed} onToggle={() => setSidebarCollapsed(!collapsed)} />
          </div>
        )}
        {!isLargeScreen && (
          <Button
            id="main-navigation-toggle"
            variant="ghost"
            size="icon"
            aria-expanded={open}
            aria-controls="main-navigation"
            aria-haspopup="menu"
            aria-label={t('sidebar.openMainNav')}
            className="size-8 rounded-full bg-dark hover:bg-dark-600"
            onClick={toggleSidebar}
          >
            <AlignJustifyIcon className="size-5! text-light" />
          </Button>
        )}
        {!isStartPage && <Breadcrumb />}
        <div className="ml-auto flex items-center gap-1">
          <ContextHelpLink />
          {!isLargeScreen && isAuthenticated && (
            <NavUserMenu email={email} side="bottom" onNavigate={closeSidebar}>
              <button
                type="button"
                aria-label={t('user.openMenu')}
                className="group flex cursor-pointer items-center gap-x-1"
              >
                <Avatar>
                  {avatarUrl && <AvatarImage src={avatarUrl} alt={`${firstName} ${lastName}`} />}
                  <AvatarFallback variant="user">
                    {`${firstName.charAt(0)}${lastName.charAt(0)}`}
                  </AvatarFallback>
                </Avatar>
                <ChevronDown className="size-5 text-dark transition-transform duration-base ease-out group-data-[state=open]:rotate-180 motion-reduce:transition-none" />
              </button>
            </NavUserMenu>
          )}
        </div>
      </div>

      <Navigation isOpen={open} closeSidebar={closeSidebar} />
    </header>
  )
}

export default Header
