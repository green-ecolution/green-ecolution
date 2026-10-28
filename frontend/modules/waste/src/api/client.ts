import { WasteBinsApi } from '@green-ecolution/backend-client'
import { configuration } from '@/api/backendApi'

export const wasteBinApi = new WasteBinsApi(configuration)

export type { WasteBinResponse as WasteBin } from '@green-ecolution/backend-client'
export type { WasteBinCreateRequest as WasteBinCreate } from '@green-ecolution/backend-client'
export type { WasteBinUpdateRequest as WasteBinUpdate } from '@green-ecolution/backend-client'
export type { WasteSettingsResponse as WasteSettings } from '@green-ecolution/backend-client'
export type { WasteSettingsUpdateRequest as WasteSettingsUpdate } from '@green-ecolution/backend-client'
