import { Link } from '@tanstack/react-router'
import { X } from 'lucide-react'
import React from 'react'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { Button } from '@green-ecolution/ui'

interface NavHeader {
  closeSidebar: () => void
  collapsed?: boolean
}

const NavHeader: React.FC<NavHeader> = ({ closeSidebar, collapsed = false }) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')

  return (
    <div
      className={`relative mb-6 flex items-start justify-between ${collapsed ? 'lg:justify-center' : ''}`}
    >
      <Link
        to="/dashboard"
        className="block transition-all ease-in-out duration-300 hover:opacity-75"
        aria-label="Zurück zum persönlichen Dashboard"
        onClick={closeSidebar}
      >
        <img
          className={`h-9 w-auto ${collapsed ? 'lg:hidden' : ''}`}
          src="/images/logo/logo-with-text-white.svg"
          alt="Logo von Green Ecolution — Smartes Grünflächenmanagement"
        />
        <img
          className={`hidden h-9 w-auto ${collapsed ? 'lg:block' : ''}`}
          src="/images/logo/logo-icon-white.svg"
          alt="Green Ecolution"
        />
      </Link>
      {!isLargeScreen && (
        <Button
          variant="ghost"
          size="icon"
          aria-label="Hauptnavigation schließen"
          className="mr-2 size-8 rounded-full bg-dark-600 hover:bg-dark-500"
          onClick={closeSidebar}
        >
          <X className="text-white" />
        </Button>
      )}
    </div>
  )
}

export default NavHeader
