import { useCallback, useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useWatch, type UseFormReturn } from 'react-hook-form'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import { organizationQueries } from '@/api/queries'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useOrganizationsWithPermission } from '@/lib/auth/useOrganizationsWithPermission'
import { useHomeOrganizationId } from '@/lib/auth/scope/effectiveRoles'
import type { TreeclusterForm } from '../../schema/treeclusterSchema'

export interface ClusterOrganizationSelection {
  /** Organizations the user may create a cluster in, empty when unknown. */
  organizations: OrganizationResponse[]
  organizationId?: string
  changeOrganization: (organizationId: string) => void
  /** Trees dropped by the last organization change; 0 once nothing was lost. */
  discardedTreeCount: number
  /** Name of any visible organization, not just a selectable one — a tree on
   *  the map may belong to one the user cannot create in. */
  nameOf: (organizationId: string) => string | undefined
  canCreateIn: (organizationId: string) => boolean
}

/**
 * Owns the organization a new cluster is created in.
 *
 * The candidate list needs the organization tree, which requires
 * `organization:read`. Without it the list stays empty, the picker is hidden
 * and no `organizationId` is sent — the backend then falls back to the
 * caller's own organization, exactly as before.
 */
export function useClusterOrganizationSelection(
  form: UseFormReturn<TreeclusterForm>,
): ClusterOrganizationSelection {
  const canReadOrganizations = useHasPermission(['organization:read'])
  const homeOrganizationId = useHomeOrganizationId()
  // `nameOf` needs every visible organization, not only the selectable ones.
  const { data: orgs } = useQuery({
    ...organizationQueries.list(),
    enabled: canReadOrganizations,
  })
  const organizations = useOrganizationsWithPermission('tree_cluster:create')

  const organizationId = useWatch({ control: form.control, name: 'organizationId' })
  const [discardedTreeCount, setDiscardedTreeCount] = useState(0)

  // Preselect the home organization (scope, else own). Falls back to the first candidate so
  // holding the right only in a sub-organization still needs no interaction.
  useEffect(() => {
    if (organizationId || organizations.length === 0) return
    const own = organizations.find((org) => org.id === homeOrganizationId)
    form.setValue('organizationId', (own ?? organizations[0]).id)
  }, [organizationId, organizations, homeOrganizationId, form])

  const changeOrganization = useCallback(
    (next: string) => {
      // Re-picking the current organization must not wipe the tree selection.
      if (!next || next === form.getValues('organizationId')) return

      const abandoned = form.getValues('treeIds') ?? []
      form.setValue('organizationId', next, { shouldValidate: true, shouldDirty: true })
      // Selection was restricted to the previous organization, so every marked
      // tree belongs to a foreign one now.
      if (abandoned.length > 0) {
        form.setValue('treeIds', [], { shouldValidate: true, shouldDirty: true })
      }
      setDiscardedTreeCount(abandoned.length)
    },
    [form],
  )

  const nameOf = useCallback((id: string) => orgs?.find((org) => org.id === id)?.name, [orgs])

  const canCreateIn = useCallback(
    (id: string) => organizations.some((org) => org.id === id),
    [organizations],
  )

  return {
    organizations,
    organizationId,
    changeOrganization,
    discardedTreeCount,
    nameOf,
    canCreateIn,
  }
}
