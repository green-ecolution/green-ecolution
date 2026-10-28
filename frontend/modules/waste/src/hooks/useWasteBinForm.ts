import { useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useWatch, type DefaultValues, type UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { wasteBinDraftResolver, type WasteBinForm } from '@green-ecolution/domain-wasm'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import { organizationQueries, userQueries } from '@/api/queries'
import { EntityFormConfig, useEntityForm } from '@/hooks/form/useEntityForm'
import { byOrgName, orgsWithPermission } from '@/lib/auth/organizationScope'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { wasteBinApi, type WasteBin, type WasteBinCreate, type WasteBinUpdate } from '../api/client'
import { wasteRoots } from '../api/roots'
import { WASTE } from '../lib/targets'

export const WASTE_BIN_FORM = 'waste-bin'

const toCreate = ({
  number,
  latitude,
  longitude,
  volumeLiters,
  depthCm,
  organizationId,
}: WasteBinForm): WasteBinCreate => ({
  number,
  latitude,
  longitude,
  volumeLiters,
  depthCm,
  organizationId,
})

// The owner changes only through a transfer, never through an edit.
const toUpdate = ({
  number,
  latitude,
  longitude,
  volumeLiters,
  depthCm,
}: WasteBinForm): WasteBinUpdate => ({ number, latitude, longitude, volumeLiters, depthCm })

export const useWasteBinForm = (
  mutationType: 'create' | 'update',
  opts: {
    wasteBinId?: string
    initForm?: DefaultValues<WasteBinForm>
    disableNavigationBlock?: boolean
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('waste')

  const config: EntityFormConfig<WasteBinForm, WasteBinForm, WasteBinForm, WasteBin> = {
    formType: WASTE_BIN_FORM,
    resolver: wasteBinDraftResolver<WasteBinForm>(translate),

    createFn: (body) => wasteBinApi.createWasteBin({ wasteBinCreateRequest: toCreate(body) }),
    updateFn: (id, body) =>
      wasteBinApi.updateWasteBin({ wasteBinId: id, wasteBinUpdateRequest: toUpdate(body) }),

    invalidates: wasteRoots.bin,

    successRoute: (id) => ({ to: '/waste-bins/$wasteBinId', params: { wasteBinId: id } }),
    replaceOnSuccess: true,
    allowedPaths: [],
    errorModule: WASTE,

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

  return useEntityForm<WasteBinForm, WasteBinForm, WasteBinForm, WasteBin>(config, mutationType, {
    entityId: opts.wasteBinId,
    initForm: opts.initForm,
    disableNavigationBlock: opts.disableNavigationBlock,
  })
}

/**
 * Organizations a new bin may be created in, with the own one preselected.
 *
 * The list needs `organization:read`; without it nothing is selected and no
 * `organizationId` is sent, so the backend falls back to the caller's own.
 */
export function useWasteBinOrganizations(
  form: UseFormReturn<WasteBinForm>,
): OrganizationResponse[] {
  const bypass = readAuthBypass()
  const canReadOrganizations = useHasPermission(['organization:read'])
  const { data: me } = useQuery(userQueries.me())
  const { data: orgs } = useQuery({
    ...organizationQueries.list(),
    enabled: canReadOrganizations,
  })

  const organizations = useMemo(() => {
    if (!orgs) return []
    // Bypass mirrors the backend's unrestricted mode: every visible org counts.
    if (bypass) return [...orgs].sort(byOrgName)
    return orgsWithPermission(orgs, me?.roles ?? [], 'waste_bin:create')
  }, [orgs, me, bypass])

  const organizationId = useWatch({ control: form.control, name: 'organizationId' })

  useEffect(() => {
    if (organizationId || organizations.length === 0) return
    const own = organizations.find((org) => org.id === me?.organization?.id)
    form.setValue('organizationId', (own ?? organizations[0]).id)
  }, [organizationId, organizations, me, form])

  return organizations
}
