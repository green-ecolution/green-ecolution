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
import NavLink from '../navigation/NavLink'
import NavHeadline from '../navigation/NavHeadline'
import NavHeader from '../navigation/NavHeader'
import useStore from '@/store/store'
import Tree from '../icons/Tree'
import SensorIcon from '../icons/Sensor'

interface NavigationProps {
  isOpen: boolean
  openSidebar: () => void
  closeSidebar: () => void
}

interface NavLinkOpts {
  id: number
  headline: string
  links: React.ReactElement[]
}

const Navigation: React.FC<NavigationProps> = ({ isOpen, openSidebar, closeSidebar }) => {
  const isLargeScreen = () => window.matchMedia('(min-width: 1024px)').matches
  const isLoggedIn = useStore((state) => state.auth.isAuthenticated)
  const mapPosition = useStore((state) => ({
    lat: state.map.center[0],
    lng: state.map.center[1],
    zoom: state.map.zoom,
  }))

  const handleMouseOver = () => {
    if (isLargeScreen()) openSidebar()
  }

  const handleMouseOut = () => {
    if (isLargeScreen()) closeSidebar()
  }

  const handleNavLinkClick = () => {
    if (!isLargeScreen()) closeSidebar()
  }

  const protectedNavLinks: NavLinkOpts[] = [
    {
      id: 1,
      headline: 'Grünflächen',
      links: [
        <NavLink
          key="nav-green-spaces-map"
          label="Karte"
          icon={<Map className="w-5 h-5" />}
          to="/map"
          search={{ lat: mapPosition.lat, lng: mapPosition.lng, zoom: mapPosition.zoom }}
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-green-spaces-clusters"
          label="Bewässerungsgruppen"
          icon={<FolderClosed className="w-5 h-5" />}
          to="/treecluster"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-green-spaces-trees"
          label="Bäume"
          icon={<Tree className="w-5 h-5" />}
          to="/trees"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
      ],
    },
    {
      id: 2,
      headline: 'Einsatzplanung',
      links: [
        <NavLink
          key="nav-watering-plans"
          label="Einsätze"
          icon={<ArrowLeftRight className="w-5 h-5" />}
          to="/watering-plans"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-watering-plan-vehicle"
          label="Fahrzeuge"
          icon={<Car className="w-5 h-5" />}
          to="/vehicles"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-more-team"
          label="Mitarbeitende"
          icon={<Users className="w-5 h-5" />}
          to="/team"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
      ],
    },
    {
      id: 3,
      headline: 'Weiteres',
      links: [
        <NavLink
          key="nav-more-sensor"
          label="Sensoren"
          icon={<SensorIcon className="w-5 h-5" />}
          to="/sensors"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-more-evaluation"
          label="Auswertung"
          icon={<PieChart className="w-5 h-5" />}
          to="/evaluations"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        <NavLink
          key="nav-more-settings"
          label="Einstellungen"
          icon={<Settings className="w-5 h-5" />}
          to="/settings"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
        />,
        // Hide the debug navigation entry in the production build
        ...(process.env.NODE_ENV !== 'production'
          ? [
              <NavLink
                key="nav-more-debug"
                label="Debug"
                icon={<Bug className="w-5 h-5" />}
                to="/debug"
                navIsOpen={isOpen}
                closeSidebar={handleNavLinkClick}
              />,
            ]
          : []),
        <NavLink
          key="nav-more-logout"
          label="Ausloggen"
          icon={<LogOut className="w-5 h-5" />}
          to="/logout"
          navIsOpen={isOpen}
          closeSidebar={handleNavLinkClick}
          preload={false}
        />,
      ],
    },
  ]

  // This is currently invisible to the user as the application is redirected directly to the login page.
  // Maybe for future use.
  const publicNavLinks: NavLinkOpts[] = [
    {
      id: 1,
      headline: '',
      links: [
        <NavLink
          key="nav-login"
          label="Anmelden"
          icon={<LogIn className="w-5 h-5" />}
          to="/login"
          navIsOpen={isOpen}
          preload={false}
          closeSidebar={handleNavLinkClick}
        />,
      ],
    },
  ]

  const navigationLinks = isLoggedIn ? protectedNavLinks : publicNavLinks

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

        {navigationLinks.map((section) => (
          <React.Fragment key={section.id}>
            <NavHeadline label={section.headline} navIsOpen={isOpen} />
            <ul className="mb-10">{section.links.map((link) => link)}</ul>
          </React.Fragment>
        ))}
      </div>
    </nav>
  )
}

export default Navigation
