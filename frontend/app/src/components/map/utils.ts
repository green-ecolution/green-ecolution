import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { WateringStatus } from '@green-ecolution/backend-client'

export const getStatusColor = (wateringStatus: WateringStatus) => {
  const statusDetails = getWateringStatusDetails(
    wateringStatus ?? WateringStatus.WateringStatusUnknown,
  )
  return statusDetails.colorHex
}
