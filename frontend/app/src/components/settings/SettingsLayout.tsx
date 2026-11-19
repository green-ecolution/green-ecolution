import type { CSSProperties, ReactNode } from 'react'
import { Link, useLocation, useNavigate } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  Bell,
  Building2,
  MapPin,
  Network,
  Puzzle,
  ShieldCheck,
  SlidersHorizontal,
  UserRound,
  Users,
} from 'lucide-react'
import { Badge } from '@green-ecolution/ui'
import { infoQueries } from '@/api/queries'
import { useAdminMode } from '@/lib/auth/scope/adminMode'
import { useSettingsHomeOrganizationId } from '@/lib/auth/scope/settingsOrganization'
import { usePermissionsIn } from '@/lib/auth/usePermissionsIn'
import { buildTree, subtreeMemberCount } from './organization/organizationTree'
import { useSettingsOrganizations } from './organization/useSettingsOrganizations'
import SettingsOrganizationPicker from './SettingsOrganizationPicker'
import { SETTINGS_NAV, visibleSettingsNav, type SettingsNavItem } from './settingsNav'
import {
  organizationIdOf,
  sectionOf,
  SECTION_ROUTES,
  type SettingsSection,
} from './settingsSections'

const ICONS = {
  UserRound,
  Building2,
  Bell,
  Users,
  Puzzle,
  MapPin,
  Network,
  ShieldCheck,
  SlidersHorizontal,
} as const

const LINK =
  'flex items-center gap-3 whitespace-nowrap rounded-xl border border-transparent px-3 py-2.5 font-nunito-sans text-sm transition-colors hover:bg-green-dark-50 data-[status=active]:border-green-dark data-[status=active]:bg-green-dark-50 data-[status=active]:font-semibold data-[status=active]:text-green-dark'
const GROUP_LABEL = 'mb-1 px-3 text-[13px] font-bold text-dark-600'
const LIST =
  '-mx-4 flex gap-2 overflow-x-auto px-4 pb-2 lg:mx-0 lg:flex-col lg:gap-1 lg:overflow-visible lg:px-0 lg:pb-0'

const isSectionRoute = (
  to: SettingsNavItem['to'],
): to is (typeof SECTION_ROUTES)[SettingsSection] =>
  Object.values<string | undefined>(SECTION_ROUTES).includes(to)

interface SettingsLayoutProps {
  children: ReactNode
}

const SettingsLayout = ({ children }: SettingsLayoutProps) => {
  const { t } = useTranslation('settings')
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const homeId = useSettingsHomeOrganizationId()
  // On the account pages the organization group still needs a target.
  const orgId = organizationIdOf(pathname) ?? homeId ?? null
  const perms = usePermissionsIn(orgId)
  const adminMode = useAdminMode()
  const { data: services } = useQuery(infoQueries.services())
  const { organizations = [], hasMemberCounts } = useSettingsOrganizations()
  const enabledFeatures = new Set(
    (services?.items ?? []).filter((item) => item.enabled).map((item) => item.name),
  )
  const items = visibleSettingsNav(SETTINGS_NAV, perms, enabledFeatures, adminMode)
  const accountItems = items.filter((item) => item.group === 'account')
  const organizationItems = orgId ? items.filter((item) => item.group === 'organization') : []

  const node = orgId ? buildTree(organizations, orgId) : null
  const countOf = (item: SettingsNavItem): number | null => {
    if (!node || !item.count) return null
    if (item.count === 'children') return node.children.length
    return hasMemberCounts ? subtreeMemberCount(node) : null
  }

  const switchOrganization = (next: string) => {
    const section = sectionOf(pathname) ?? 'overview'
    void navigate({ to: SECTION_ROUTES[section], params: { orgId: next } })
  }

  const renderItem = (item: SettingsNavItem) => {
    const Icon = ICONS[item.icon as keyof typeof ICONS]
    const count = countOf(item)
    const content = (
      <>
        <Icon className="size-5 shrink-0" aria-hidden />
        <span className={item.comingSoon ? 'text-dark-500' : undefined}>
          {t(`nav.${item.key}`)}
        </span>
        {count !== null && count > 0 && (
          <span className="ml-auto hidden text-[13px] text-dark-600 lg:inline">{count}</span>
        )}
        {item.comingSoon && (
          <Badge variant="muted" className="ml-auto hidden lg:inline-flex">
            {t('layout.comingSoonBadge')}
          </Badge>
        )}
      </>
    )
    return (
      <li key={item.key} className="shrink-0 lg:shrink">
        {orgId && isSectionRoute(item.to) ? (
          <Link
            to={item.to}
            params={{ orgId }}
            activeOptions={{ exact: item.key === 'overview' }}
            className={LINK}
          >
            {content}
          </Link>
        ) : (
          <Link to={item.to} className={LINK}>
            {content}
          </Link>
        )}
      </li>
    )
  }

  return (
    <div
      className="container mt-6 pb-16"
      style={
        {
          '--app-header-h': '4.5625rem',
          '--settings-header-top': 'calc(4.5625rem + 3.5rem)',
        } as CSSProperties
      }
    >
      <div className="sticky top-[var(--app-header-h)] z-30 -mx-4 mb-6 flex h-14 items-center bg-light px-4 lg:mx-0 lg:px-0">
        <h1 className="font-lato text-3xl font-bold lg:text-4xl">{t('layout.title')}</h1>
      </div>

      <div className="lg:grid lg:grid-cols-[272px_1fr] lg:gap-8">
        <nav
          aria-label={t('layout.navAriaLabel')}
          className="z-20 flex flex-col gap-4 lg:sticky lg:top-[var(--settings-header-top)] lg:gap-7 lg:self-start"
        >
          <div>
            <p className={GROUP_LABEL}>{t('layout.groupAccount')}</p>
            <ul className={LIST}>{accountItems.map(renderItem)}</ul>
          </div>

          {orgId && organizationItems.length > 0 && (
            <div>
              <p className={GROUP_LABEL}>{t('layout.groupOrganization')}</p>
              <div className="mb-2">
                <SettingsOrganizationPicker
                  organizations={organizations}
                  selectedId={orgId}
                  onSelect={switchOrganization}
                />
              </div>
              <ul className={LIST}>{organizationItems.map(renderItem)}</ul>
            </div>
          )}
        </nav>

        <div className="mt-6 rounded-2xl bg-dark-50 p-4 lg:mt-0 lg:p-6">{children}</div>
      </div>
    </div>
  )
}

export default SettingsLayout
