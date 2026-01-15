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
import { useCallback, useMemo } from 'react'
import { useShallow } from 'zustand/react/shallow'
import { LinkProps } from '@tanstack/react-router'
import NavLink from '../navigation/NavLink'
import NavHeadline from '../navigation/NavHeadline'
import NavHeader from '../navigation/NavHeader'
import useStore from '@/store/store'
import Tree from '../icons/Tree'
import SensorIcon from '../icons/Sensor'
import { useMediaQuery } from '@/hooks/useMediaQuery'

interface NavigationProps {
  isOpen: boolean
  openSidebar: () => void
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

const Navigation: React.FC<NavigationProps> = ({ isOpen, openSidebar, closeSidebar }) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const isLoggedIn = useStore((state) => state.isAuthenticated)
  const mapPosition = useStore(
    useShallow((state) => ({
      lat: state.mapCenter[0],
      lng: state.mapCenter[1],
      zoom: state.mapZoom,
    })),
  )

  const handleMouseOver = useCallback(() => {
    if (isLargeScreen) openSidebar()
  }, [isLargeScreen, openSidebar])

  const handleMouseOut = useCallback(() => {
    if (isLargeScreen) closeSidebar()
  }, [isLargeScreen, closeSidebar])

  const handleNavLinkClick = useCallback(() => {
    if (!isLargeScreen) closeSidebar()
  }, [isLargeScreen, closeSidebar])

  const protectedNavData: NavSectionData[] = useMemo(
    () => [
      {
        id: 1,
        headline: 'Grünflächen',
        links: [
          {
            key: 'nav-green-spaces-map',
            label: 'Karte',
            icon: <Map className="w-5 h-5" />,
            to: '/map',
            search: { lat: mapPosition.lat, lng: mapPosition.lng, zoom: mapPosition.zoom },
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
          {
            key: 'nav-more-settings',
            label: 'Einstellungen',
            icon: <Settings className="w-5 h-5" />,
            to: '/settings',
          },
          // Hide the debug navigation entry in the production build
          ...(process.env.NODE_ENV !== 'production'
            ? [
                {
                  key: 'nav-more-debug',
                  label: 'Debug',
                  icon: <Bug className="w-5 h-5" />,
                  to: '/debug',
                } as NavLinkData,
              ]
            : []),
          {
            key: 'nav-more-logout',
            label: 'Ausloggen',
            icon: <LogOut className="w-5 h-5" />,
            to: '/logout',
            preload: false,
          },
        ],
      },
    ],
    [mapPosition.lat, mapPosition.lng, mapPosition.zoom],
  )

  const navigationData = isLoggedIn ? protectedNavData : publicNavData

  return (
    <nav
      id="main-navigation"
      aria-label="Hauptnavigation"
      onMouseOut={handleMouseOut}
      onMouseOver={handleMouseOver}
      className={`fixed inset-0 z-50 bg-dark w-screen overflow-hidden h-screen transition-all ease-in-out duration-300
        ${isOpen ? 'visible block left-0 lg:w-[17rem] lg:rounded-r-xl' : 'invisible -left-full lg:visible lg:w-[5rem] lg:left-0'}`}
    >
      <div className="relative px-4 py-5 h-full overflow-y-auto no-scrollbar">
        <NavHeader isOpen={isOpen} closeSidebar={closeSidebar} />

        {navigationData.map((section) => (
          <React.Fragment key={section.id}>
            <NavHeadline label={section.headline} navIsOpen={isOpen} />
            <ul className="mb-10">
              {section.links.map(({ key, label, icon, ...linkProps }) => (
                <NavLink
                  key={key}
                  label={label}
                  icon={icon}
                  navIsOpen={isOpen}
                  closeSidebar={handleNavLinkClick}
                  {...linkProps}
                />
              ))}
            </ul>
          </React.Fragment>
        ))}
      </div>
    </nav>
  )
}

export default Navigation
