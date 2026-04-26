import { useMutation, useQueryClient } from '@tanstack/react-query'
import { vehicleIdQuery, vehicleQuery } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { useNavigate } from '@tanstack/react-router'
import type { Vehicle, VehicleCreate, VehicleUpdate } from '@/api/backendApi'
import { vehicleApi } from '@/api/backendApi'
import { VehicleForm, vehicleSchema } from '@/schema/vehicleSchema'
import { DefaultValues, useForm } from 'react-hook-form'
import { zodResolver } from '@/lib/zodResolver'

export const useVehicleForm = (
  mutationType: 'create' | 'update',
  opts: { vehicleId?: string; initForm?: DefaultValues<VehicleForm> },
) => {
  const showToast = createToast()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const form = useForm<VehicleForm>({
    defaultValues: opts.initForm,
    resolver: zodResolver(vehicleSchema),
  })

  const { mutate, isError, error } = useMutation({
    mutationFn: (vehicle: VehicleCreate | VehicleUpdate) => {
      if (mutationType === 'create') {
        return vehicleApi.createVehicle({
          vehicleCreateRequest: vehicle as VehicleCreate,
        })
      } else if (mutationType === 'update' && opts.vehicleId) {
        return vehicleApi.updateVehicle({
          vehicleId: Number(opts.vehicleId),
          vehicleUpdateRequest: vehicle as VehicleUpdate,
        })
      }
      return Promise.reject(Error('Invalid mutation type or missing vehicleId for update'))
    },

    onSuccess: (data: Vehicle) => {
      queryClient
        .invalidateQueries(vehicleIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "vehicleIdQuery" failed:', error))
      queryClient
        .invalidateQueries(vehicleQuery())
        .catch((error) => console.error('Invalidate "vehicleQuery" failed', error))

      navigate({
        to: `/vehicles/$vehicleId`,
        params: { vehicleId: data.id.toString() },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      if (mutationType === 'create') showToast('Das Fahrzeug wurde erfolgreich erstellt.')
      else showToast('Das Fahrzeug wurde erfolgreich bearbeitet.')
    },

    onError: (error) => {
      console.error('Error with vehicle mutation:', error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error')
    },
    throwOnError: true,
  })

  return {
    mutate,
    isError,
    error,
    form,
  }
}
