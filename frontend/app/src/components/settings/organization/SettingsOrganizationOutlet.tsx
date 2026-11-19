import { getRouteApi, Outlet, useLocation } from '@tanstack/react-router'
import Forbidden from '@/components/layout/Forbidden'
import { satisfies } from '@/lib/auth/permissions'
import { usePermissionsLoadError, usePermissionsResolver } from '@/lib/auth/usePermissionsIn'
import { sectionPermission } from '../settingsNav'
import { sectionOf } from '../settingsSections'

const organizationRoute = getRouteApi('/_protected/settings/organization/$orgId')

// The route guards only know the union of the user's rights; this checks the
// selected organization, which a typed address or the picker can change.
const SettingsOrganizationOutlet = () => {
  const { orgId } = organizationRoute.useParams()
  const { pathname } = useLocation()
  const permissionsOf = usePermissionsResolver()
  const loadError = usePermissionsLoadError()
  if (!permissionsOf) {
    // Thrown to the route's error view rather than waiting on a request that failed.
    if (loadError) throw loadError
    return null
  }

  const required = sectionPermission(sectionOf(pathname))
  if (required && !satisfies(permissionsOf(orgId), required)) return <Forbidden />

  // The organization picker navigates between the same pages of other organizations;
  // the key remounts the page so filters, selection and drafts do not leak across.
  return <Outlet key={orgId} />
}

export default SettingsOrganizationOutlet
