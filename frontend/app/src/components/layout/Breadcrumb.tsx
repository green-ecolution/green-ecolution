import { Link } from '@tanstack/react-router'
import { useBreadcrumbs } from '@/hooks/useBreadcrumb'
import {
  Breadcrumb as BreadcrumbRoot,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@green-ecolution/ui'

function Breadcrumb() {
  const breadcrumbs = useBreadcrumbs()

  const rootBreadcrumb = {
    title: 'Dashboard',
    path: '/dashboard',
  }

  const isLastItem = (index: number) => index === breadcrumbs.length - 1

  return (
    <BreadcrumbRoot aria-label="Seitennavigation" className="hidden lg:block">
      <BreadcrumbList>
        <BreadcrumbItem>
          <BreadcrumbLink asChild>
            <Link to={rootBreadcrumb.path}>{rootBreadcrumb.title}</Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        {breadcrumbs.map((breadcrumb, index) => (
          <BreadcrumbItem key={breadcrumb.path}>
            <BreadcrumbSeparator />
            {isLastItem(index) ? (
              <BreadcrumbPage>{breadcrumb.title}</BreadcrumbPage>
            ) : (
              <BreadcrumbLink asChild>
                <Link to={breadcrumb.path}>{breadcrumb.title}</Link>
              </BreadcrumbLink>
            )}
          </BreadcrumbItem>
        ))}
      </BreadcrumbList>
    </BreadcrumbRoot>
  )
}

export default Breadcrumb
