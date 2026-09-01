import { Link } from '@tanstack/react-router'
import { X } from 'lucide-react'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { Button } from '@green-ecolution/ui'

interface NavHeader {
  closeSidebar: () => void
  collapsed?: boolean
}

const NavHeader: React.FC<NavHeader> = ({ closeSidebar, collapsed = false }) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const { t } = useTranslation('navigation')

  return (
    <div
      className={`relative mb-6 flex items-start justify-between ${collapsed ? 'lg:justify-center' : ''}`}
    >
      <Link
        to="/dashboard"
        className="block transition-opacity duration-quick ease-out hover:opacity-75"
        aria-label={t('sidebar.backToDashboard')}
        onClick={closeSidebar}
      >
        <img
          className={`h-9 w-auto ${collapsed ? 'lg:hidden' : ''}`}
          src="/images/logo/logo-with-text-white.svg"
          alt={t('sidebar.logoAlt')}
        />
        <img
          className={`hidden h-9 w-auto ${collapsed ? 'lg:block' : ''}`}
          src="/images/logo/logo-icon-white.svg"
          alt={t('sidebar.logoIconAlt')}
        />
      </Link>
      {!isLargeScreen && (
        <Button
          variant="ghost"
          size="icon"
          aria-label={t('sidebar.closeMainNav')}
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
