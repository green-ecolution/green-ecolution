import { AlignJustifyIcon } from 'lucide-react'
import { useState, useCallback } from 'react'
import Navigation from './Navigation'
import Breadcrumb from './Breadcrumb'
import ProfileButton from './ProfileButton'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { Button } from '@green-ecolution/ui'

function Header() {
  const [open, setOpen] = useState(false)
  const isStartPage = location.pathname === '/'
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')

  const openSidebar = useCallback(() => {
    setOpen(true)
    if (!isLargeScreen) {
      document.body.classList.add('overflow-y-hidden')
    }
  }, [isLargeScreen])

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

  return (
    <header className="relative z-10 bg-white lg:pl-20">
      <div className="container text-sm border-b border-dark-50 py-4 flex justify-between items-center">
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
        <ProfileButton />
      </div>

      <Navigation isOpen={open} openSidebar={openSidebar} closeSidebar={closeSidebar} />
    </header>
  )
}

export default Header
