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
import { Link } from '@tanstack/react-router'
import { ArrowRight, Database, Droplets, Radio, TreeDeciduous, Trees, Truck } from 'lucide-react'
import { Bar, BarChart, XAxis, YAxis, Cell } from 'recharts'
import HeroStatCard from './HeroStatCard'

const dataChartConfig: ChartConfig = {
  value: {
    label: 'Anzahl',
    color: 'hsl(142, 76%, 36%)',
  },
}

interface DataTabContentProps {
  statsData:
    | {
        treeCount?: number
        treeClusterCount?: number
        sensorCount?: number
        vehicleCount?: number
        wateringPlanCount?: number
      }
    | undefined
}

const DataTabContent = ({ statsData }: DataTabContentProps) => {
  const chartData = [
    {
      name: 'Bäume',
      value: statsData?.treeCount ?? 0,
      fill: 'hsl(142, 76%, 36%)',
    },
    {
      name: 'Gruppen',
      value: statsData?.treeClusterCount ?? 0,
      fill: 'hsl(142, 60%, 45%)',
    },
    {
      name: 'Sensoren',
      value: statsData?.sensorCount ?? 0,
      fill: 'hsl(200, 70%, 50%)',
    },
    {
      name: 'Fahrzeuge',
      value: statsData?.vehicleCount ?? 0,
      fill: 'hsl(35, 90%, 55%)',
    },
    {
      name: 'Pläne',
      value: statsData?.wateringPlanCount ?? 0,
      fill: 'hsl(210, 60%, 55%)',
    },
  ]

  const totalEntities =
    (statsData?.treeCount ?? 0) +
    (statsData?.treeClusterCount ?? 0) +
    (statsData?.sensorCount ?? 0) +
    (statsData?.vehicleCount ?? 0) +
    (statsData?.wateringPlanCount ?? 0)

  return (
    <div className="space-y-6">
      {/* Hero stat with chart */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Main tree count - featured */}
        <HeroStatCard
          className="lg:col-span-1"
          gradient="bg-gradient-to-br from-green-dark/5 to-transparent"
          icon={<TreeDeciduous className="size-8 text-green-dark" />}
          iconBoxClassName="p-3 bg-green-dark/10"
          footer={
            <Link
              to="/treecluster"
              search={{ page: 1 }}
              className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
            >
              Alle Baumgruppen ansehen
              <ArrowRight className="size-4" />
            </Link>
          }
        >
          <div>
            <p className="text-sm font-medium text-dark-600 mb-1">Bäume gesamt</p>
            <p className="text-5xl font-bold font-lato text-green-dark tracking-tight">
              {statsData?.treeCount?.toLocaleString('de-DE') ?? '-'}
            </p>
            <p className="text-sm text-dark-500 mt-2">
              in {statsData?.treeClusterCount?.toLocaleString('de-DE') ?? '-'} Gruppen verwaltet
            </p>
          </div>
        </HeroStatCard>

        {/* Chart overview */}
        <Card className="lg:col-span-2">
          <CardHeader className="pb-2">
            <CardTitle className="text-base font-medium">Datenübersicht</CardTitle>
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
        <DataStatCard
          icon={<Trees className="size-5" />}
          label="Baumgruppen"
          value={statsData?.treeClusterCount ?? 0}
          subtitle="Verwaltete Cluster"
          color="green"
          href="/treecluster"
        />
        <DataStatCard
          icon={<Radio className="size-5" />}
          label="Sensoren"
          value={statsData?.sensorCount ?? 0}
          subtitle="Aktive Messgeräte"
          color="blue"
          href="/sensors"
        />
        <DataStatCard
          icon={<Truck className="size-5" />}
          label="Fahrzeuge"
          value={statsData?.vehicleCount ?? 0}
          subtitle="Registrierte Flotte"
          color="orange"
          href="/vehicles"
        />
        <DataStatCard
          icon={<Droplets className="size-5" />}
          label="Bewässerungspläne"
          value={statsData?.wateringPlanCount ?? 0}
          subtitle="Geplante Routen"
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
                <p className="text-sm font-medium">Gesamte Datensätze</p>
                <p className="text-xs text-dark-500">Alle verwalteten Entitäten</p>
              </div>
            </div>
            <p className="text-2xl font-bold font-lato">{totalEntities.toLocaleString('de-DE')}</p>
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
  const styles = colorStyles[color]

  return (
    <Link to={href} className="block group">
      <Card className="h-full transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <CardContent className="pt-5 pb-4">
          <div className="flex items-start gap-3">
            <div className={`p-2.5 rounded-lg ${styles.bg}`}>
              <span className={styles.text}>{icon}</span>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm text-dark-600 font-medium truncate">{label}</p>
              <p className="text-2xl font-bold font-lato mt-0.5">{value.toLocaleString('de-DE')}</p>
              <p className="text-xs text-dark-400 mt-1">{subtitle}</p>
            </div>
          </div>
          <div className="mt-3 flex items-center gap-1.5 text-xs font-medium text-dark-500 group-hover:text-green-dark transition-colors">
            Ansehen
            <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5" />
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}

export default DataTabContent
