import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@green-ecolution/ui'
import type { DataStatisticsResponse } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import { ArrowRight, Database, Droplets, Radio, Truck } from 'lucide-react'
import { Bar, BarChart, XAxis, YAxis, Cell } from 'recharts'
import { useTranslation } from 'react-i18next'
import { useNumberFormatter } from '@/lib/i18n/useFormatters'
import { planningManifests } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'
import type { I18nKey } from '@/modules/types'
import HeroStatCard from './HeroStatCard'

// Bar colors for module statistics in manifest order; the first two are the former tree and group bars.
const MODULE_FILLS = [
  'hsl(142, 76%, 36%)',
  'hsl(142, 60%, 45%)',
  'hsl(170, 55%, 40%)',
  'hsl(100, 45%, 45%)',
]

interface DataTabContentProps {
  statsData: DataStatisticsResponse | undefined
}

const DataTabContent = ({ statsData }: DataTabContentProps) => {
  const { t } = useTranslation('info')
  const { i18n } = useTranslation()
  const numberFormatter = useNumberFormatter()
  const { manifests } = useActiveModules()

  const translate = (key: I18nKey, options?: { count: string }) => i18n.t(key, options)
  const moduleCount = (module: string, kind: string) =>
    statsData?.modules.find((m) => m.module === module && m.kind === kind)?.count
  const moduleStatistics = manifests.flatMap((manifest) =>
    Object.entries(manifest.statistics ?? {}).map(([kind, statistic]) => ({
      id: `${manifest.key}.${kind}`,
      module: manifest.key,
      kind,
      statistic,
      count: moduleCount(manifest.key, kind),
    })),
  )
  const planning = planningManifests(manifests)
  const plansLabelKey = planning.length === 1 ? planning[0].planning.plansLabelKey : undefined
  const hero = moduleStatistics.find((entry) => entry.statistic.hero)
  const heroCompanion = hero?.statistic.hero
  const heroCompanionCount =
    hero && heroCompanion ? moduleCount(hero.module, heroCompanion.companionKind) : undefined

  const dataChartConfig: ChartConfig = {
    value: {
      label: t('data.chart.countLabel'),
      color: 'hsl(142, 76%, 36%)',
    },
  }

  const chartData = [
    ...moduleStatistics.map((entry, index) => ({
      name: translate(entry.statistic.chartLabelKey ?? entry.statistic.labelKey),
      value: entry.count ?? 0,
      fill: MODULE_FILLS[index % MODULE_FILLS.length],
    })),
    {
      name: t('data.chart.sensorName'),
      value: statsData?.sensorCount ?? 0,
      fill: 'hsl(200, 70%, 50%)',
    },
    {
      name: t('data.chart.vehicleName'),
      value: statsData?.vehicleCount ?? 0,
      fill: 'hsl(35, 90%, 55%)',
    },
    {
      name: t('data.chart.planName'),
      value: statsData?.planCount ?? 0,
      fill: 'hsl(210, 60%, 55%)',
    },
  ]

  const totalEntities =
    moduleStatistics.reduce((sum, entry) => sum + (entry.count ?? 0), 0) +
    (statsData?.sensorCount ?? 0) +
    (statsData?.vehicleCount ?? 0) +
    (statsData?.planCount ?? 0)

  return (
    <div className="space-y-6">
      {/* Hero stat with chart */}
      <div className="grid gap-6 lg:grid-cols-3">
        {hero && heroCompanion && (
          <HeroStatCard
            className="lg:col-span-1"
            gradient="bg-gradient-to-br from-green-dark/5 to-transparent"
            icon={<hero.statistic.icon className="size-8 text-green-dark" />}
            iconBoxClassName="p-3 bg-green-dark/10"
            footer={
              hero.statistic.to && (
                <Link
                  to={hero.statistic.to}
                  search={{ page: 1 }}
                  className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
                >
                  {translate(heroCompanion.linkLabelKey)}
                  <ArrowRight className="size-4" />
                </Link>
              )
            }
          >
            <div>
              <p className="text-sm font-medium text-dark-600 mb-1">
                {translate(hero.statistic.labelKey)}
              </p>
              <p className="text-5xl font-bold font-lato text-green-dark tracking-tight">
                {hero.count != null ? numberFormatter.format(hero.count) : '-'}
              </p>
              <p className="text-sm text-dark-500 mt-2">
                {translate(heroCompanion.companionKey, {
                  count:
                    heroCompanionCount != null ? numberFormatter.format(heroCompanionCount) : '-',
                })}
              </p>
            </div>
          </HeroStatCard>
        )}

        {/* Chart overview */}
        <Card className={hero ? 'lg:col-span-2' : 'lg:col-span-3'}>
          <CardHeader className="pb-2">
            <CardTitle className="text-base font-medium">{t('data.chart.title')}</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={dataChartConfig} className="h-[180px] w-full">
              <BarChart data={chartData} layout="vertical" margin={{ left: 0, right: 20 }}>
                <XAxis type="number" hide />
                <YAxis
                  type="category"
                  dataKey="name"
                  axisLine={false}
                  tickLine={false}
                  tick={{ fontSize: 12 }}
                  width={70}
                />
                <ChartTooltip
                  content={<ChartTooltipContent />}
                  cursor={{ fill: 'hsl(var(--muted))' }}
                />
                <Bar dataKey="value" radius={[0, 4, 4, 0]} barSize={24}>
                  {chartData.map((entry) => (
                    <Cell key={entry.name} fill={entry.fill} />
                  ))}
                </Bar>
              </BarChart>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>

      {/* Secondary stats grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {moduleStatistics.map(
          ({ id, statistic, count }) =>
            statistic.subtitleKey &&
            statistic.to && (
              <DataStatCard
                key={id}
                icon={<statistic.icon className="size-5" />}
                label={translate(statistic.labelKey)}
                value={count ?? 0}
                subtitle={translate(statistic.subtitleKey)}
                color="green"
                href={statistic.to}
              />
            ),
        )}
        <DataStatCard
          icon={<Radio className="size-5" />}
          label={t('data.stat.sensorsLabel')}
          value={statsData?.sensorCount ?? 0}
          subtitle={t('data.stat.sensorsSubtitle')}
          color="blue"
          href="/sensors"
        />
        <DataStatCard
          icon={<Truck className="size-5" />}
          label={t('data.stat.vehiclesLabel')}
          value={statsData?.vehicleCount ?? 0}
          subtitle={t('data.stat.vehiclesSubtitle')}
          color="orange"
          href="/vehicles"
        />
        <DataStatCard
          icon={<Droplets className="size-5" />}
          label={plansLabelKey ? translate(plansLabelKey) : t('data.stat.plansLabel')}
          value={statsData?.planCount ?? 0}
          subtitle={t('data.stat.plansSubtitle')}
          color="cyan"
          href="/watering-plans"
        />
      </div>

      {/* Summary footer */}
      <Card className="bg-dark-100/50">
        <CardContent className="py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-dark-200 rounded-lg">
                <Database className="size-4 text-dark-600" />
              </div>
              <div>
                <p className="text-sm font-medium">{t('data.summary.totalLabel')}</p>
                <p className="text-xs text-dark-500">{t('data.summary.totalHint')}</p>
              </div>
            </div>
            <p className="text-2xl font-bold font-lato">{numberFormatter.format(totalEntities)}</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface DataStatCardProps {
  icon: React.ReactNode
  label: string
  value: number
  subtitle: string
  color: 'green' | 'blue' | 'orange' | 'cyan'
  href: string
}

const colorStyles = {
  green: {
    bg: 'bg-green-dark/10',
    text: 'text-green-dark',
    accent: 'bg-green-dark',
  },
  blue: {
    bg: 'bg-blue-500/10',
    text: 'text-blue-600',
    accent: 'bg-blue-500',
  },
  orange: {
    bg: 'bg-orange-500/10',
    text: 'text-orange-600',
    accent: 'bg-orange-500',
  },
  cyan: {
    bg: 'bg-cyan-500/10',
    text: 'text-cyan-600',
    accent: 'bg-cyan-500',
  },
}

function DataStatCard({ icon, label, value, subtitle, color, href }: DataStatCardProps) {
  const { t } = useTranslation('info')
  const numberFormatter = useNumberFormatter()
  const styles = colorStyles[color]

  return (
    <Link to={href} className="block group">
      <Card className="h-full transition-[box-shadow,translate] duration-base ease-out hover:shadow-md hover:-translate-y-0.5 motion-reduce:transition-none">
        <CardContent className="pt-5 pb-4">
          <div className="flex items-start gap-3">
            <div className={`p-2.5 rounded-lg ${styles.bg}`}>
              <span className={styles.text}>{icon}</span>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm text-dark-600 font-medium truncate">{label}</p>
              <p className="text-2xl font-bold font-lato mt-0.5">{numberFormatter.format(value)}</p>
              <p className="text-xs text-dark-400 mt-1">{subtitle}</p>
            </div>
          </div>
          <div className="mt-3 flex items-center gap-1.5 text-xs font-medium text-dark-500 group-hover:text-green-dark transition-colors">
            {t('data.stat.viewLink')}
            <ArrowRight className="size-3 transition-transform duration-base ease-emphasized group-hover:translate-x-0.5 motion-reduce:transition-none" />
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}

export default DataTabContent
