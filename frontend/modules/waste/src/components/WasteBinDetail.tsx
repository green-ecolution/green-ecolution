import { useId, useState, type ReactNode } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronDown, Pencil, Trash2 } from 'lucide-react'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import {
  Button,
  ButtonGroup,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  DetailedList,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'
import CommentsSection from '@/components/comments/CommentsSection'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import EntityDetailHeader from '@/components/general/EntityDetailHeader'
import createToast from '@/hooks/createToast'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { wasteBinApi, type WasteBin } from '../api/client'
import { wasteRoots } from '../api/roots'
import { binTarget } from '../lib/targets'
import FillStateBadge from './FillStateBadge'

interface WasteBinDetailProps {
  bin: WasteBin
}

const DetailCard = ({ title, children }: { title: string; children: ReactNode }) => {
  const headingId = useId()
  return (
    <section aria-labelledby={headingId}>
      <Card variant="outlined" className="h-full">
        <CardHeader>
          <CardTitle asChild>
            <h2 id={headingId}>{title}</h2>
          </CardTitle>
        </CardHeader>
        <CardContent>{children}</CardContent>
      </Card>
    </section>
  )
}

const WasteBinDetail = ({ bin }: WasteBinDetailProps) => {
  const { t } = useTranslation('waste')
  const dateLocale = useDateLocale()
  const navigate = useNavigate()
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  const [confirmDelete, setConfirmDelete] = useState(false)
  const canEdit = useHasPermission(['waste_bin:update'])
  const canDelete = useHasPermission(['waste_bin:delete'])

  const formatDateTime = (value: string) =>
    format(new Date(value), 'dd.MM.yyyy, HH:mm', { locale: dateLocale })

  const handleDelete = () => {
    wasteBinApi
      .deleteWasteBin({ wasteBinId: bin.id })
      // Invalidate only after leaving: this page holds a live query on the bin,
      // which would refetch into a 404 while still mounted.
      .then(() => navigate({ to: '/waste-bins', search: { page: 1 } }))
      .then(() => invalidate(wasteRoots.bin))
      .then(() => showToast(t('detail.deleteSuccessToast')))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast(t('detail.deleteErrorToast'), 'error')
      })
  }

  const masterData = [
    { label: t('detail.masterData.numberLabel'), value: bin.number },
    {
      label: t('detail.masterData.volumeLabel'),
      value: t('units.liters', { value: bin.volumeLiters }),
    },
    {
      label: t('detail.masterData.depthLabel'),
      value: t('units.centimeters', { value: bin.depthCm }),
    },
    {
      label: t('detail.masterData.coordinatesLabel'),
      value: `${bin.latitude.toFixed(6)}, ${bin.longitude.toFixed(6)}`,
    },
    {
      label: t('detail.masterData.dataSourceLabel'),
      value: bin.provider ?? t('detail.masterData.manuallyCreated'),
    },
    {
      label: t('detail.masterData.lastUpdateLabel'),
      value: format(new Date(bin.updatedAt), 'dd.MM.yyyy', { locale: dateLocale }),
    },
  ]

  const fill = [
    {
      label: t('detail.fill.stateLabel'),
      value: <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />,
    },
    {
      label: t('detail.fill.levelLabel'),
      value:
        bin.fillPercent != null
          ? t('units.percent', { value: bin.fillPercent })
          : t('detail.fill.noMeasurement'),
    },
    {
      label: t('detail.fill.lastMeasurementLabel'),
      value: bin.fillMeasuredAt
        ? formatDateTime(bin.fillMeasuredAt)
        : t('detail.fill.noMeasurement'),
    },
    {
      label: t('detail.fill.lastEmptiedLabel'),
      value: bin.lastEmptiedAt ? formatDateTime(bin.lastEmptiedAt) : t('detail.fill.neverEmptied'),
    },
    {
      label: t('detail.fill.sensorLabel'),
      value: bin.sensorId ? (
        <Link
          to="/sensors/$sensorId"
          params={{ sensorId: bin.sensorId }}
          className="break-all text-green-dark underline hover:text-green-light"
        >
          {bin.sensorId}
        </Link>
      ) : (
        t('detail.fill.noSensor')
      ),
    },
  ]

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/waste-bins' }, label: t('detail.backToList') }}
        title={<>{t('detail.title', { number: bin.number })}</>}
        badge={<FillStateBadge state={bin.fillState} percent={bin.fillPercent} size="lg" />}
        actions={
          (canEdit || canDelete) && (
            <ButtonGroup>
              {canEdit && (
                <Button variant="outline" asChild>
                  <Link to="/waste-bins/$wasteBinId/edit" params={{ wasteBinId: bin.id }}>
                    {t('detail.editLink')}
                    <Pencil className="stroke-1" />
                  </Link>
                </Button>
              )}
              {canDelete && (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      aria-label={t('detail.moreActionsAriaLabel')}
                      className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-base data-[state=open]:[&_svg]:rotate-180 motion-reduce:[&_svg]:transition-none"
                    >
                      <ChevronDown />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="min-w-[12rem]">
                    <DropdownMenuItem
                      className="gap-2 px-3 py-2 text-destructive focus:bg-destructive/10 focus:text-destructive"
                      onSelect={() => setConfirmDelete(true)}
                    >
                      <Trash2 />
                      {t('detail.deleteMenuItem')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </ButtonGroup>
          )
        }
      />

      <div className="mt-10 grid gap-6 lg:grid-cols-2">
        <DetailCard title={t('detail.masterData.title')}>
          <DetailedList details={masterData} columns={1} />
        </DetailCard>
        <DetailCard title={t('detail.fill.title')}>
          <DetailedList details={fill} columns={1} />
        </DetailCard>
      </div>

      <div className="mt-6">
        <CommentsSection subject={{ kind: 'target', target: binTarget(bin.id) }} />
      </div>

      <DeleteConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={t('detail.deleteDialogTitle', { number: bin.number })}
        description={t('detail.deleteDialogDescription')}
        onConfirm={handleDelete}
      />
    </>
  )
}

export default WasteBinDetail
