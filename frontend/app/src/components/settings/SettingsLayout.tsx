import type { CSSProperties, ReactNode } from 'react'
import { Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { Bell, Building2, Droplet, Map, Puzzle, RadioTower, UserRound, Users } from 'lucide-react'
import { Badge } from '@green-ecolution/ui'
import { servicesInfoQuery } from '@/api/queries'
import { usePermissions } from '@/lib/auth/usePermissions'
import { SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

const ICONS = { UserRound, Building2, Droplet, Bell, RadioTower, Users, Map, Puzzle } as const

interface SettingsLayoutProps {
  children: ReactNode
}

const SettingsLayout = ({ children }: SettingsLayoutProps) => {
  const perms = usePermissions()
  const { data: services } = useQuery(servicesInfoQuery())
  const enabledFeatures = new Set(
    (services?.items ?? []).filter((item) => item.enabled).map((item) => item.name),
  )
  const items = visibleSettingsNav(SETTINGS_NAV, perms, enabledFeatures)

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
      <div className="sticky top-[var(--app-header-h)] z-30 mb-6 flex h-14 items-center bg-white">
        <h1 className="font-lato text-3xl font-bold lg:text-4xl">Einstellungen</h1>
      </div>

      <div className="lg:grid lg:grid-cols-[240px_1fr] lg:gap-8">
        <nav
          aria-label="Einstellungsbereiche"
          className="z-20 lg:sticky lg:top-[var(--settings-header-top)] lg:self-start"
        >
          <ul className="-mx-4 flex gap-2 overflow-x-auto px-4 pb-2 lg:mx-0 lg:flex-col lg:overflow-visible lg:px-0 lg:pb-0">
            {items.map((item) => {
              const Icon = ICONS[item.icon as keyof typeof ICONS]
              return (
                <li key={item.key} className="shrink-0 lg:shrink">
                  <Link
                    to={item.to}
                    activeOptions={{ exact: false }}
                    className="flex items-center gap-3 whitespace-nowrap rounded-xl border border-transparent px-3 py-2.5 font-nunito-sans text-sm transition-colors hover:bg-green-dark-50 data-[status=active]:border-green-dark data-[status=active]:bg-green-dark-50 data-[status=active]:font-semibold data-[status=active]:text-green-dark"
                  >
                    <Icon className="size-5 shrink-0" aria-hidden />
                    <span className={item.comingSoon ? 'text-dark-500' : undefined}>
                      {item.label}
                    </span>
                    {item.comingSoon && (
                      <Badge variant="muted" className="ml-auto hidden lg:inline-flex">
                        bald
                      </Badge>
                    )}
                  </Link>
                </li>
              )
            })}
          </ul>
        </nav>

        <div className="mt-6 rounded-2xl bg-dark-50 p-4 lg:mt-0 lg:p-6">{children}</div>
      </div>
    </div>
  )
}

export default SettingsLayout
