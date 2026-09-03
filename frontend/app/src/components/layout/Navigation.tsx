import {
  ArrowLeftRight,
  BookOpen,
  Bug,
  Car,
  FolderClosed,
  LogIn,
  Map,
  PieChart,
  Settings,
} from 'lucide-react'
import * as React from 'react'
import { useCallback } from 'react'
import { LinkProps } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import NavLink from '../navigation/NavLink'
import NavHeadline from '../navigation/NavHeadline'
import NavHeader from '../navigation/NavHeader'
import NavUser from '../navigation/NavUser'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { useCurrentUserAvatar } from '@/lib/auth/useCurrentUserAvatar'
import Tree from '../icons/Tree'
import SensorIcon from '../icons/Sensor'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'
import { visibleNavSections } from '@/lib/auth/routePermissions'
import { usePermissions } from '@/lib/auth/usePermissions'

interface NavigationProps {
  isOpen: boolean
  closeSidebar: () => void
}

interface NavLinkData extends LinkProps {
  key: string
  label: string
  icon: React.ReactNode
}

interface NavSectionData {
  id: number
  headline: string
  links: NavLinkData[]
}

const publicNavData = (t: TFunction<'navigation'>): NavSectionData[] => [
  {
    id: 1,
    headline: '',
    links: [
      {
        key: 'nav-login',
        label: t('sidebar.login'),
        icon: <LogIn className="w-5 h-5" />,
        to: '/login',
        preload: false,
      },
    ],
  },
]

// Settings and the user entry are pinned to the bottom of the sidebar,
// separate from the navigation sections.
const footerNavData = (t: TFunction<'navigation'>): NavLinkData[] => [
  {
    key: 'nav-settings',
    label: t('sidebar.settings'),
    icon: <Settings className="w-5 h-5" />,
    to: '/settings',
  },
]

const protectedNavData = (t: TFunction<'navigation'>): NavSectionData[] => [
  {
    id: 1,
    headline: t('sidebar.headlineGreenSpaces'),
    links: [
      {
        key: 'nav-green-spaces-map',
        label: t('sidebar.map'),
        icon: <Map className="w-5 h-5" />,
        to: '/map',
        preload: false,
      },
      {
        key: 'nav-green-spaces-clusters',
        label: t('sidebar.clusters'),
        icon: <FolderClosed className="w-5 h-5" />,
        to: '/treecluster',
      },
      {
        key: 'nav-green-spaces-trees',
        label: t('sidebar.trees'),
        icon: <Tree className="w-5 h-5" />,
        to: '/trees',
      },
    ],
  },
  {
    id: 2,
    headline: t('sidebar.headlinePlanning'),
    links: [
      {
        key: 'nav-watering-plans',
        label: t('sidebar.wateringPlans'),
        icon: <ArrowLeftRight className="w-5 h-5" />,
        to: '/watering-plans',
      },
      {
        key: 'nav-watering-plan-vehicle',
        label: t('sidebar.vehicles'),
        icon: <Car className="w-5 h-5" />,
        to: '/vehicles',
      },
    ],
  },
  {
    id: 3,
    headline: t('sidebar.headlineMore'),
    links: [
      {
        key: 'nav-more-help',
        label: t('sidebar.help'),
        icon: <BookOpen className="w-5 h-5" />,
        to: '/help',
      },
      {
        key: 'nav-more-sensor',
        label: t('sidebar.sensors'),
        icon: <SensorIcon className="w-5 h-5" />,
        to: '/sensors',
      },
      {
        key: 'nav-more-evaluation',
        label: t('sidebar.evaluations'),
        icon: <PieChart className="w-5 h-5" />,
        to: '/evaluations',
      },
      // Hide the debug navigation entry in the production build
      ...(import.meta.env.DEV
        ? [
            {
              key: 'nav-more-debug',
              label: t('sidebar.debug'),
              icon: <Bug className="w-5 h-5" />,
              to: '/debug',
            } as NavLinkData,
          ]
        : []),
    ],
  },
]

const Navigation: React.FC<NavigationProps> = ({ isOpen, closeSidebar }) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const { isAuthenticated: isLoggedIn } = useAuthSession()
  const collapsed = useSidebarCollapsed()
  const { t } = useTranslation('navigation')

  const { firstName, lastName, email } = useCurrentUser()
  const avatarUrl = useCurrentUserAvatar()

  const handleNavLinkClick = useCallback(() => {
    if (!isLargeScreen) closeSidebar()
  }, [isLargeScreen, closeSidebar])

  const perms = usePermissions()
  const navigationData = isLoggedIn
    ? visibleNavSections(protectedNavData(t), perms)
    : publicNavData(t)

  return (
    <nav
      id="main-navigation"
      aria-label={t('sidebar.mainNavLabel')}
      className={`fixed inset-0 z-50 bg-dark w-screen h-dvh flex flex-col ease-in-out duration-300 transition-[left,width,visibility] motion-reduce:transition-none
        lg:left-0 lg:visible ${collapsed ? 'lg:w-[4.5rem]' : 'lg:w-[16rem]'}
        ${isOpen ? 'visible left-0' : 'invisible -left-full'}`}
    >
      <div className={`shrink-0 px-4 pt-5 ${collapsed ? 'lg:px-2' : ''}`}>
        <NavHeader closeSidebar={closeSidebar} collapsed={collapsed} />
      </div>

      <div
        className={`flex min-h-0 flex-1 flex-col overflow-y-auto no-scrollbar px-4 ${collapsed ? 'lg:px-2' : ''}`}
      >
        {navigationData.map((section) => (
          <React.Fragment key={section.id}>
            <NavHeadline label={section.headline} collapsed={collapsed} />
            <ul className="mb-6 space-y-1">
              {section.links.map(({ key, label, icon, ...linkProps }) => (
                <NavLink
                  key={key}
                  label={label}
                  icon={icon}
                  collapsed={collapsed}
                  closeSidebar={handleNavLinkClick}
                  {...linkProps}
                />
              ))}
            </ul>
          </React.Fragment>
        ))}
      </div>

      {/* Collapse toggle, settings and user entry are pinned below the
          scrollable section list. The collapse toggle only exists from lg up;
          logged-out users still get it, so the block hides on mobile when
          logged out. */}
      <div
        className={`shrink-0 border-t border-dark-400/30 px-4 pb-4 pt-4 ${collapsed ? 'lg:px-2' : ''} ${isLoggedIn ? '' : 'hidden lg:block'}`}
      >
        <ul className="space-y-1">
          {isLoggedIn &&
            footerNavData(t).map(({ key, label, icon, ...linkProps }) => (
              <NavLink
                key={key}
                label={label}
                icon={icon}
                collapsed={collapsed}
                closeSidebar={handleNavLinkClick}
                {...linkProps}
              />
            ))}
        </ul>
        {/* On mobile the user menu lives in the header instead */}
        {isLoggedIn && (
          <div className="mt-2 hidden lg:block">
            <NavUser
              firstName={firstName}
              lastName={lastName}
              email={email}
              avatarUrl={avatarUrl}
              collapsed={collapsed}
              closeSidebar={handleNavLinkClick}
            />
          </div>
        )}
      </div>
    </nav>
  )
}

export default Navigation
