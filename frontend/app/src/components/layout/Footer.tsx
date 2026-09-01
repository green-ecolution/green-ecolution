import { useLocation } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { infoQueries } from '@/api/queries'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'

function Footer() {
  const location = useLocation()
  const isMapPage = location.pathname.includes('/map')
  const collapsed = useSidebarCollapsed()
  const { data: appInfo } = useQuery(infoQueries.app())
  const { t } = useTranslation('navigation')

  const version = appInfo?.version?.startsWith('v')
    ? appInfo.version
    : `v${appInfo?.version ?? t('footer.versionUnknown')}`

  const navItems = [
    {
      url: 'mailto:info@green-ecolution.de',
      label: t('footer.contact'),
    },
    {
      url: 'https://green-ecolution.de/impressum',
      label: t('footer.imprint'),
    },
    {
      url: 'https://green-ecolution.de/datenschutz',
      label: t('footer.privacy'),
    },
  ]

  return (
    <footer
      className={`bg-white transition-[padding] ease-in-out duration-300 motion-reduce:transition-none mt-16 ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'} ${isMapPage ? 'hidden' : ''}`}
    >
      <div className="container text-sm border-t border-dark-50 py-4 lg:flex lg:justify-between lg:items-center">
        <p className="text-dark-400 mb-5 lg:mb-0">{t('footer.tagline', { version })}</p>
        <nav aria-label={t('footer.nav')}>
          <ul className="flex flex-wrap gap-x-4">
            {navItems.map((navItem) => (
              <li key={navItem.url}>
                <a
                  href={navItem.url}
                  target="_blank"
                  className="text-dark-600 transition-colors duration-quick ease-out hover:text-dark-800"
                >
                  {navItem.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </footer>
  )
}

export default Footer
