import type { SelectedTarget } from '@/modules/types'
import { binTarget } from '../../lib/targets'

export const binSelection = (bin: { id: string; number: string }): SelectedTarget => ({
  target: binTarget(bin.id),
  label: bin.number,
})
