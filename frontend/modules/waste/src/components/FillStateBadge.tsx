import type { ComponentType } from 'react'
import { CircleAlert, CircleHelp, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, cn, type BadgeProps } from '@green-ecolution/ui'
import type { FillStateDto } from '@green-ecolution/backend-client'

interface FillStateBadgeProps {
  state: FillStateDto
  percent?: number | null
  size?: BadgeProps['size']
  className?: string
}

const display: Record<
  FillStateDto,
  { variant: BadgeProps['variant']; icon: ComponentType<{ className?: string }>; dashed?: boolean }
> = {
  full: { variant: 'error', icon: CircleAlert },
  normal: { variant: 'success', icon: Trash2 },
  empty: { variant: 'muted', icon: Trash2 },
  unknown: { variant: 'muted', icon: CircleHelp, dashed: true },
}

const FillStateBadge = ({ state, percent, size, className }: FillStateBadgeProps) => {
  const { t } = useTranslation('waste')
  const { variant, icon: Icon, dashed } = display[state]
  const label = t(`fillState.${state}`)
  // An emptying after the last reading makes that reading's percentage stale.
  const showPercent = percent != null && (state === 'full' || state === 'normal')

  return (
    <Badge
      variant={variant}
      size={size}
      data-fill-state={state}
      className={cn('inline-flex items-center gap-1.5', dashed && 'border-dashed', className)}
    >
      <Icon aria-hidden className="size-3.5" />
      <span>{showPercent ? t('fillState.withPercent', { state: label, percent }) : label}</span>
    </Badge>
  )
}

export default FillStateBadge
