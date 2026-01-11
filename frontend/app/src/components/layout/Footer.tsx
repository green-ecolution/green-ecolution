import { useLocation } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { infoQuery } from '@/api/queries'

function Footer() {
  const location = useLocation()
  const isMapPage = location.pathname.includes('/map')
  const { data: appInfo } = useQuery(infoQuery())

  const version = appInfo?.version?.startsWith('v')
    ? appInfo.version
    : `v${appInfo?.version ?? 'unkown'}`

  const navItems = [
    {
      url: 'mailto:info@green-ecolution.de',
      label: 'Kontakt',
    },
    {
      url: 'https://green-ecolution.de/impressum',
      label: 'Impressum',
    },
    {
      url: 'https://green-ecolution.de/datenschutz',
      label: 'Datenschutz',
    },
  ]

  return (
    <footer className={`bg-white lg:pl-20 mt-16 ${isMapPage ? 'hidden' : ''}`}>
      <div className="container text-sm border-t border-dark-50 py-4 lg:flex lg:justify-between lg:items-center">
        <p className="text-dark-400 mb-5 lg:mb-0">
          Smartes Grünflächenmanagement - Green Ecolution {version}
        </p>
        <nav aria-label="Fußnavigation">
          <ul className="flex flex-wrap gap-x-4">
            {navItems.map((navItem) => (
              <li key={navItem.url}>
                <a
                  href={navItem.url}
                  target="_blank"
                  className="text-dark-600 transition-all ease-in-out hover:text-dark-800"
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
