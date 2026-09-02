import { Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@green-ecolution/ui'
import { ArrowUpRight, Link2, Link2Off, TreeDeciduous } from 'lucide-react'
import { treeQueries } from '@/api/queries'
import type { Sensor } from '@/api/backendApi'
import { useSensorActions } from './SensorActionsContext'

interface SensorLinkedTreeSectionProps {
  sensor: Sensor
}

const SensorLinkedTreeSection = ({ sensor }: SensorLinkedTreeSectionProps) => {
  const { t } = useTranslation('sensor')
  const hasLink = sensor.linkedTreeId != null
  const treeIdStr = hasLink ? String(sensor.linkedTreeId) : ''
  const { data: tree, isLoading, isError } = useQuery(treeQueries.detail(treeIdStr))
  const actions = useSensorActions()

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            {hasLink ? <Link2 className="size-5" /> : <Link2Off className="size-5" />}
          </div>
          <CardTitle>{t('linkedTree.title')}</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        {!hasLink ? (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>{t('linkedTree.noLinkTitle')}</AlertTitle>
                <AlertDescription>{t('linkedTree.noLinkDescription')}</AlertDescription>
                <div className="mt-3">
                  <Button
                    variant="outline"
                    size="sm"
                    className="gap-2 [&_svg]:size-4"
                    onClick={() => actions.requestActivate()}
                  >
                    <Link2 />
                    {t('actions.activateAssignTree')}
                  </Button>
                </div>
              </AlertContent>
            </div>
          </Alert>
        ) : isLoading ? (
          <div className="h-24 rounded-2xl bg-dark-50 animate-pulse" />
        ) : isError || !tree ? (
          <Alert variant="destructive">
            <div className="flex gap-3">
              <AlertIcon variant="destructive" />
              <AlertContent>
                <AlertTitle>{t('linkedTree.loadFailedTitle')}</AlertTitle>
                <AlertDescription>
                  {t('linkedTree.loadFailedDescription', { id: sensor.linkedTreeId })}
                </AlertDescription>
              </AlertContent>
            </div>
          </Alert>
        ) : (
          <Link
            to="/trees/$treeId"
            params={{ treeId: String(tree.id) }}
            className="group relative flex items-center gap-5 rounded-2xl border border-dark-100 bg-white p-5 transition hover:border-green-dark hover:shadow-cards"
          >
            <div className="grid place-items-center size-14 rounded-xl bg-green-dark text-white shrink-0">
              <TreeDeciduous className="size-7" strokeWidth={1.75} />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs uppercase tracking-widest text-muted-foreground">
                {t('linkedTree.treeNumberLabel', { number: tree.number })}
              </p>
              <p className="font-lato font-bold text-xl mt-1 truncate">
                {tree.species || t('linkedTree.unknownSpecies')}
              </p>
              <p className="text-sm text-muted-foreground mt-0.5">
                {t('linkedTree.plantedLabel', { year: tree.plantingYear })}
              </p>
            </div>
            <ArrowUpRight className="size-5 text-muted-foreground group-hover:text-green-dark transition shrink-0" />
          </Link>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorLinkedTreeSection
