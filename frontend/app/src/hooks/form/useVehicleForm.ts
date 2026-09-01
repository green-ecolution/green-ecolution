import type { Vehicle, VehicleCreate, VehicleUpdate } from '@/api/backendApi'
import { vehicleApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { VehicleForm } from '@/schema/vehicleSchema'
import { vehicleDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
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
      createLeave:
        'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen des Fahrzeugs gehen verloren, wenn du jetzt gehst.',
      updateLeave:
        'Möchtest du die Seite wirklich verlassen? Deine Änderungen am Fahrzeug gehen verloren, wenn du jetzt gehst.',
      createSuccess: 'Das Fahrzeug wurde erfolgreich erstellt.',
      updateSuccess: 'Das Fahrzeug wurde erfolgreich bearbeitet.',
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
