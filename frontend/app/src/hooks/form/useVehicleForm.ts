import { VEHICLE_FORM } from '@/store/form/formDraftSlice'
import { queryRoots } from '@/api/queries'
import type { Vehicle } from '@/api/backendApi'
import { vehicleApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { VehicleForm } from '@/schema/vehicleSchema'
import { vehicleDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const FORM_CAPACITY_KINDS = ['water', 'cargo_volume']

const toRequest = (
  { waterCapacity, cargoVolumeCapacity, ...fields }: VehicleForm,
  capacities: Record<string, number> = {},
) => {
  const untouched = Object.fromEntries(
    Object.entries(capacities).filter(([kind]) => !FORM_CAPACITY_KINDS.includes(kind)),
  )
  return {
    ...fields,
    capacities: {
      ...untouched,
      ...(waterCapacity !== undefined ? { water: waterCapacity } : {}),
      ...(cargoVolumeCapacity !== undefined ? { cargo_volume: cargoVolumeCapacity } : {}),
    },
  }
}

export const useVehicleForm = (
  mutationType: 'create' | 'update',
  opts: {
    vehicleId?: string
    initForm?: DefaultValues<VehicleForm>
    /** Capacities of the loaded vehicle; kinds the form does not edit are sent back unchanged. */
    capacities?: Record<string, number>
    disableNavigationBlock?: boolean
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('vehicle')

  const vehicleConfig: EntityFormConfig<VehicleForm, VehicleForm, VehicleForm, Vehicle> = {
    formType: VEHICLE_FORM,
    resolver: vehicleDraftResolver<VehicleForm>(translate),

    createFn: (body) => vehicleApi.createVehicle({ vehicleCreateRequest: toRequest(body) }),
    updateFn: (id, body) =>
      vehicleApi.updateVehicle({
        vehicleId: id,
        vehicleUpdateRequest: toRequest(body, opts.capacities),
      }),

    // Watering plans embed their transporter and trailer.
    invalidates: [...queryRoots.vehicle, ...queryRoots.wateringPlan],

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
