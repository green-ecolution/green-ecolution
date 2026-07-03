import {
  ArrowLeftRight,
  Bug,
  Car,
  FolderClosed,
  LogIn,
  LogOut,
  Map,
  PieChart,
  Settings,
  Users,
} from 'lucide-react'
import * as React from 'react'
import { useCallback } from 'react'
import { LinkProps } from '@tanstack/react-router'
import NavLink from '../navigation/NavLink'
import NavHeadline from '../navigation/NavHeadline'
import NavHeader from '../navigation/NavHeader'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import Tree from '../icons/Tree'
import SensorIcon from '../icons/Sensor'
import { useMediaQuery } from '@/hooks/useMediaQuery'

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

const publicNavData: NavSectionData[] = [
  {
    id: 1,
    headline: '',
    links: [
      {
        key: 'nav-login',
        label: 'Anmelden',
        icon: <LogIn className="w-5 h-5" />,
        to: '/login',
        preload: false,
      },
    ],
  },
]

// Settings and logout are pinned to the bottom of the sidebar, separate from
// the navigation sections.
const footerNavData: NavLinkData[] = [
  {
    key: 'nav-settings',
    label: 'Einstellungen',
    icon: <Settings className="w-5 h-5" />,
    to: '/settings',
  },
  {
    key: 'nav-logout',
    label: 'Abmelden',
    icon: <LogOut className="w-5 h-5" />,
    to: '/logout',
    preload: false,
  },
]

const protectedNavData: NavSectionData[] = [
  {
    id: 1,
    headline: 'Grünflächen',
    links: [
      {
        key: 'nav-green-spaces-map',
        label: 'Karte',
        icon: <Map className="w-5 h-5" />,
        to: '/map',
        preload: false,
      },
      {
        key: 'nav-green-spaces-clusters',
        label: 'Bewässerungsgruppen',
        icon: <FolderClosed className="w-5 h-5" />,
        to: '/treecluster',
      },
      {
        key: 'nav-green-spaces-trees',
        label: 'Bäume',
        icon: <Tree className="w-5 h-5" />,
        to: '/trees',
      },
    ],
  },
  {
    id: 2,
    headline: 'Einsatzplanung',
    links: [
      {
        key: 'nav-watering-plans',
        label: 'Einsätze',
        icon: <ArrowLeftRight className="w-5 h-5" />,
        to: '/watering-plans',
      },
      {
        key: 'nav-watering-plan-vehicle',
        label: 'Fahrzeuge',
        icon: <Car className="w-5 h-5" />,
        to: '/vehicles',
      },
      {
        key: 'nav-more-team',
        label: 'Mitarbeitende',
        icon: <Users className="w-5 h-5" />,
        to: '/team',
      },
    ],
  },
  {
    id: 3,
    headline: 'Weiteres',
    links: [
      {
        key: 'nav-more-sensor',
        label: 'Sensoren',
        icon: <SensorIcon className="w-5 h-5" />,
        to: '/sensors',
      },
      {
        key: 'nav-more-evaluation',
        label: 'Auswertung',
        icon: <PieChart className="w-5 h-5" />,
        to: '/evaluations',
      },
      // Hide the debug navigation entry in the production build
      ...(import.meta.env.DEV
        ? [
            {
              key: 'nav-more-debug',
              label: 'Debug',
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

  const handleNavLinkClick = useCallback(() => {
    if (!isLargeScreen) closeSidebar()
  }, [isLargeScreen, closeSidebar])

  const navigationData = isLoggedIn ? protectedNavData : publicNavData

  return (
    <nav
      id="main-navigation"
      aria-label="Hauptnavigation"
      className={`fixed inset-0 z-50 bg-dark w-screen h-screen flex flex-col ease-in-out duration-300 transition-[left,visibility]
        lg:left-0 lg:w-[4.5rem] lg:visible xl:w-[16rem]
        ${isOpen ? 'visible left-0' : 'invisible -left-full'}`}
    >
      <div className="shrink-0 px-4 pt-5 lg:px-2 xl:px-4">
        <NavHeader closeSidebar={closeSidebar} />
      </div>

      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto no-scrollbar px-4 lg:px-2 xl:px-4">
        {navigationData.map((section) => (
          <React.Fragment key={section.id}>
            <NavHeadline label={section.headline} />
            <ul className="mb-6 space-y-1">
              {section.links.map(({ key, label, icon, ...linkProps }) => (
                <NavLink
                  key={key}
                  label={label}
                  icon={icon}
                  closeSidebar={handleNavLinkClick}
                  {...linkProps}
                />
              ))}
            </ul>
          </React.Fragment>
        ))}

        {/* Settings + logout: pinned to the bottom on desktop (lg:mt-auto),
            inline at the end of the scrollable list on the mobile overlay. */}
        {isLoggedIn && (
          <div className="pb-4 lg:mt-auto lg:border-t lg:border-dark-400/30 lg:pt-4">
            <ul className="space-y-1">
              {footerNavData.map(({ key, label, icon, ...linkProps }) => (
                <NavLink
                  key={key}
                  label={label}
                  icon={icon}
                  closeSidebar={handleNavLinkClick}
                  {...linkProps}
                />
              ))}
            </ul>
          </div>
        )}
      </div>
    </nav>
  )
}

export default Navigation
