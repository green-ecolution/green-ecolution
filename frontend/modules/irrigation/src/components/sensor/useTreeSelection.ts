import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import type { SelectedTarget } from '@/modules/types'
import { treeTarget } from '../../lib/targets'

interface SelectableTree {
  id: string
  number: string
  species: string
}

export const useTreeSelection = () => {
  const { t } = useTranslation('irrigation')
  return useCallback(
    (tree: SelectableTree): SelectedTarget => ({
      target: treeTarget(tree.id),
      label: t('sensor.wizard.treeLabel', { number: tree.number }),
      detail: tree.species || undefined,
    }),
    [t],
  )
}
