import { VehicleType, DrivingLicense, VehicleAvailability } from '@green-ecolution/backend-client'
import { VEHICLE_FORM } from '@/store/form/formDraftSlice'
import { queryRoots } from '@/api/queries'
import type { Vehicle } from '@/api/backendApi'
import { vehicleApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { vehicleDraftResolver, type VehicleForm } from '@/schema/vehicleSchema'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'
import type { ModuleCapacityKind } from '@/modules/types'

// Every field the form can register needs a key here, the optional ones included
// (capacities through `vehicleCapacityDefaults`): a registered field missing from
// the defaults marks the untouched form as dirty.
export const NEW_VEHICLE_DEFAULTS: DefaultValues<VehicleForm> = {
  numberPlate: '',
  model: '',
  description: '',
  type: VehicleType.Transporter,
  drivingLicense: DrivingLicense.B,
  availability: VehicleAvailability.Available,
  height: 2.5,
  width: 2,
  length: 6,
  weight: 3.5,
}

/** Stored amounts stay in the form state, so a kind no module offers goes back unchanged. */
export const vehicleCapacityDefaults = (
  offered: readonly ModuleCapacityKind[],
  stored: Record<string, number> = {},
): Record<string, number | undefined> => ({
  ...Object.fromEntries(offered.map(({ kind }) => [kind, undefined])),
  ...stored,
})

const toRequest = ({ capacities, ...fields }: VehicleForm) => ({
  ...fields,
  capacities: Object.fromEntries(
    Object.entries(capacities ?? {}).filter(
      (entry): entry is [string, number] => typeof entry[1] === 'number',
    ),
  ),
})

export const useVehicleForm = (
  mutationType: 'create' | 'update',
  opts: {
    vehicleId?: string
    initForm?: DefaultValues<VehicleForm>
    /** The capacity kinds the form shows and validates. */
    offered?: readonly ModuleCapacityKind[]
    disableNavigationBlock?: boolean
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('vehicle')

  const vehicleConfig: EntityFormConfig<VehicleForm, VehicleForm, VehicleForm, Vehicle> = {
    formType: VEHICLE_FORM,
    resolver: vehicleDraftResolver<VehicleForm>(translate, opts.offered ?? []),

    createFn: (body) => vehicleApi.createVehicle({ vehicleCreateRequest: toRequest(body) }),
    updateFn: (id, body) =>
      vehicleApi.updateVehicle({
        vehicleId: id,
        vehicleUpdateRequest: toRequest(body),
      }),

    // Plans embed their transporter and trailer.
    invalidates: [...queryRoots.vehicle, ...queryRoots.plan],

    successRoute: (id) => ({
      to: '/vehicles/$vehicleId',
      params: { vehicleId: id.toString() },
    }),
    replaceOnSuccess: true,
    allowedPaths: [],

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

  return useEntityForm<VehicleForm, VehicleForm, VehicleForm, Vehicle>(
    vehicleConfig,
    mutationType,
    {
      entityId: opts.vehicleId,
      initForm: opts.initForm,
      disableNavigationBlock: opts.disableNavigationBlock,
    },
  )
}
