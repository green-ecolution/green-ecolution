import type { Vehicle, VehicleCreate, VehicleUpdate } from '@/api/backendApi'
import { vehicleApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { VehicleForm } from '@/schema/vehicleSchema'
import { vehicleDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

export const useVehicleForm = (
  mutationType: 'create' | 'update',
  opts: {
    vehicleId?: string
    initForm?: DefaultValues<VehicleForm>
    disableNavigationBlock?: boolean
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('vehicle')

  const vehicleConfig: EntityFormConfig<VehicleForm, VehicleCreate, VehicleUpdate, Vehicle> = {
    formType: 'vehicle',
    resolver: vehicleDraftResolver<VehicleForm>(translate),

    createFn: (body) => vehicleApi.createVehicle({ vehicleCreateRequest: body }),
    updateFn: (id, body) => vehicleApi.updateVehicle({ vehicleId: id, vehicleUpdateRequest: body }),

    // Watering plans embed their transporter and trailer.
    invalidates: ['vehicle', 'wateringPlan'],

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

  return useEntityForm<VehicleForm, VehicleCreate, VehicleUpdate, Vehicle>(
    vehicleConfig,
    mutationType,
    {
      entityId: opts.vehicleId,
      initForm: opts.initForm,
      disableNavigationBlock: opts.disableNavigationBlock,
    },
  )
}
