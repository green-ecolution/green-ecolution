import { Link } from '@tanstack/react-router'
import { ChevronRight } from 'lucide-react'
import { useBreadcrumbs } from '@/hooks/useBreadcrumb'

function Breadcrumb() {
  const breadcrumbs = useBreadcrumbs()

  const rootBreadcrumb = {
    title: 'Dashboard',
    path: '/dashboard',
  }

  return (
    <nav
      aria-label="Seitennavigation"
      className="hidden text-xs text-dark-800 lg:block data-[status=active]:text-dark transition-all ease-in-out duration-300 hover:text-green-dark hover:data-[status=active]:text-green-dark"
    >
      <ul className="flex flex-wrap items-center">
        <li className="flex items-center">
          <Link to={rootBreadcrumb.path}>{rootBreadcrumb.title}</Link>
        </li>
        {breadcrumbs.map((breadcrumb) => (
          <li
            key={breadcrumb.path}
            className="flex items-center data-[status=active]:text-dark transition-all ease-in-out duration-300 hover:text-green-dark hover:data-[status=active]:text-green-dark last:font-semibold"
          >
            <ChevronRight className="w-3.5 stroke-1" />
            <Link to={breadcrumb.path}>{breadcrumb.title}</Link>
          </li>
        ))}
      </ul>
    </nav>
  )
}

export default Breadcrumb
