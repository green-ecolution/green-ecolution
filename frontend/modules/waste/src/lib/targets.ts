import type { TargetRefDto } from '@green-ecolution/backend-client'

export const WASTE = 'waste'
export const BIN_KIND = 'bin'

export const binTarget = (binId: string): TargetRefDto => ({
  module: WASTE,
  kind: BIN_KIND,
  id: binId,
})
