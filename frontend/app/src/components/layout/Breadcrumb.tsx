import { Fragment } from 'react'
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
          <Fragment key={breadcrumb.path}>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              {isLastItem(index) ? (
                <BreadcrumbPage>{breadcrumb.title}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink asChild>
                  <Link to={breadcrumb.path}>{breadcrumb.title}</Link>
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
          </Fragment>
        ))}
      </BreadcrumbList>
    </BreadcrumbRoot>
  )
}

export default Breadcrumb
