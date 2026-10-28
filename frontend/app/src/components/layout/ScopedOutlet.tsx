import { Outlet } from '@tanstack/react-router'
import { useOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'

// Forms and local state were filled under the previous scope; a scope switch
// remounts the subtree instead of carrying them over.
function ScopedOutlet() {
  const scope = useOrganizationScope()
  return <Outlet key={scope ?? ''} />
}

export default ScopedOutlet
