import { getRouteApi, Outlet } from '@tanstack/react-router'

const organizationRoute = getRouteApi('/_protected/settings/organization/$orgId')

const SettingsOrganizationOutlet = () => {
  const { orgId } = organizationRoute.useParams()
  // The organization picker navigates between the same pages of other organizations;
  // the key remounts the page so filters, selection and drafts do not leak across.
  return <Outlet key={orgId} />
}

export default SettingsOrganizationOutlet
