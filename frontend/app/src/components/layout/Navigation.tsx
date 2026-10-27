import { ArrowLeftRight, BookOpen, Bug, Car, LogIn, Map, Puzzle, Settings } from 'lucide-react'
import * as React from 'react'
import { useCallback } from 'react'
import { LinkProps } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import NavLink from '../navigation/NavLink'
import NavHeadline from '../navigation/NavHeadline'
import NavHeader from '../navigation/NavHeader'
import NavUser from '../navigation/NavUser'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { useCurrentUserAvatar } from '@/lib/auth/useCurrentUserAvatar'
import SensorIcon from '../icons/Sensor'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'
import {
  planResourcesOf,
  routePermissionsFor,
  visibleNavSections,
} from '@/lib/auth/routePermissions'
import { useActiveModulesOptional } from '@/modules/useActiveModules'
import type { I18nKey, ModuleManifest, NavSectionKey } from '@/modules/types'
import { usePermissions } from '@/lib/auth/usePermissions'
import { pluginViewsQuery } from '@/api/queries'
import type { PluginNavEntryResponse } from '@/api/backendApi'

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
  key: NavSectionKey | 'public' | 'plugins'
  headline: string
  links: NavLinkData[]
}

const publicNavData = (t: TFunction<'navigation'>): NavSectionData[] => [
  {
    key: 'public',
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

interface CoreSection {
  key: NavSectionKey
  headline: string
  before: NavLinkData[]
  after: NavLinkData[]
}

const moduleLinksFor = (
  section: NavSectionKey,
  manifests: readonly ModuleManifest[],
  translate: (key: I18nKey) => string,
): NavLinkData[] =>
  manifests.flatMap((manifest) =>
    (manifest.nav?.links ?? [])
      .filter((link) => link.section === section)
      .map((link) => ({
        key: link.key,
        label: translate(link.labelKey),
        icon: <link.icon className="w-5 h-5" />,
        to: link.to as NavLinkData['to'],
      })),
  )

// Module links sit between `before` and `after`, where the irrigation entries used to be.
const protectedNavData = (
  t: TFunction<'navigation'>,
  translate: (key: I18nKey) => string,
  manifests: readonly ModuleManifest[],
): NavSectionData[] => {
  const sections: CoreSection[] = [
    {
      key: 'greenspaces',
      headline: t('sidebar.headlineGreenSpaces'),
      // Without a module drawing on it the map is an empty basemap.
      before: manifests.some(({ map }) => map)
        ? [
            {
              key: 'nav-green-spaces-map',
              label: t('sidebar.map'),
              icon: <Map className="w-5 h-5" />,
              to: '/map',
              preload: false,
            },
          ]
        : [],
      after: [],
    },
    {
      key: 'planning',
      headline: t('sidebar.headlinePlanning'),
      before:
        planResourcesOf(manifests).length > 0
          ? [
              {
                key: 'nav-watering-plans',
                label: t('sidebar.wateringPlans'),
                icon: <ArrowLeftRight className="w-5 h-5" />,
                to: '/plans',
              },
            ]
          : [],
      after: [
        {
          key: 'nav-watering-plan-vehicle',
          label: t('sidebar.vehicles'),
          icon: <Car className="w-5 h-5" />,
          to: '/vehicles',
        },
      ],
    },
    {
      key: 'analysis',
      headline: t('sidebar.headlineMore'),
      before: [
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
      ],
      after: [
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

  return sections.map(({ key, headline, before, after }) => ({
    key,
    headline,
    links: [...before, ...moduleLinksFor(key, manifests, translate), ...after],
  }))
}

const pluginNavSection = (
  t: TFunction<'navigation'>,
  views: PluginNavEntryResponse[],
): NavSectionData => ({
  key: 'plugins',
  headline: t('sidebar.headlinePlugins'),
  links: views.map((view) => ({
    key: `nav-plugin-${view.slug}`,
    label: view.name,
    icon: <Puzzle className="w-5 h-5" />,
    to: '/plugin/$slug',
    params: { slug: view.slug },
  })),
})

interface NavSectionsProps {
  sections: NavSectionData[]
  collapsed: boolean
  onLinkClick: () => void
}

const NavSections: React.FC<NavSectionsProps> = ({ sections, collapsed, onLinkClick }) =>
  sections.map((section) => (
    <React.Fragment key={section.key}>
      <NavHeadline label={section.headline} collapsed={collapsed} />
      <ul className="mb-6 space-y-1">
        {section.links.map(({ key, label, icon, ...linkProps }) => (
          <NavLink
            key={key}
            label={label}
            icon={icon}
            collapsed={collapsed}
            closeSidebar={onLinkClick}
            {...linkProps}
          />
        ))}
      </ul>
    </React.Fragment>
  ))

// Split out so the active-modules query only runs for a logged-in user.
const ProtectedNavSections: React.FC<Omit<NavSectionsProps, 'sections'>> = (props) => {
  const { t } = useTranslation('navigation')
  const { i18n } = useTranslation()
  const { data: pluginViews = [] } = useQuery(pluginViewsQuery())
  const perms = usePermissions()
  const { manifests } = useActiveModulesOptional()
  const sections = visibleNavSections(
    [...protectedNavData(t, (key) => i18n.t(key), manifests), pluginNavSection(t, pluginViews)],
    perms,
    routePermissionsFor(manifests),
  )

  return <NavSections sections={sections} {...props} />
}

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
        {isLoggedIn ? (
          <ProtectedNavSections collapsed={collapsed} onLinkClick={handleNavLinkClick} />
        ) : (
          <NavSections
            sections={publicNavData(t)}
            collapsed={collapsed}
            onLinkClick={handleNavLinkClick}
          />
        )}
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
