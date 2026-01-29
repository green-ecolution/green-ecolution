import { infoQuery, serverInfoQuery, servicesInfoQuery, statisticsQuery } from '@/api/queries'
import { RuntimeStats } from '@/components/info/RuntimeStats'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Loading,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
  toast,
} from '@green-ecolution/ui'
import { useSuspenseQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute, Link, useSearch } from '@tanstack/react-router'
import { z } from 'zod'
import {
  Activity,
  ArrowRight,
  ArrowUp,
  CheckCircle2,
  Clock,
  Code,
  Copy,
  Database,
  Droplets,
  ExternalLink,
  FlaskConical,
  GitBranch,
  GitCommit,
  Globe,
  HardDrive,
  Layers,
  Monitor,
  Network,
  Package,
  Radio,
  Server,
  Settings,
  Shield,
  Tag,
  TreeDeciduous,
  Trees,
  Truck,
  XCircle,
  Zap,
} from 'lucide-react'
import type { ServiceStatus, VersionInfo } from '@green-ecolution/backend-client'
import { Bar, BarChart, XAxis, YAxis, Cell } from 'recharts'

const tabSchema = z.enum(['system', 'data', 'software', 'server', 'runtime']).catch('system')

export const Route = createFileRoute('/_protected/info')({
  component: Info,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Lade Systeminformationen" />
  ),
  validateSearch: z.object({
    tab: tabSchema.default('system'),
  }),
  loader: ({ context: { queryClient } }) => {
    Promise.all([
      queryClient.prefetchQuery(infoQuery()),
      queryClient.prefetchQuery(servicesInfoQuery()),
    ]).catch((error) => console.error('Prefetching info queries failed', error))
    return {
      crumb: {
        title: 'Systeminformationen',
      },
    }
  },
})

const serviceNameMap: Record<string, string> = {
  database: 'Datenbank',
  auth: 'Authentifizierung',
  mqtt: 'MQTT',
  s3: 'S3 Speicher',
  routing: 'Routing (Valhalla)',
  vroom: 'Routenoptimierung (Vroom)',
}

function getServiceDisplayName(name: string): string {
  return serviceNameMap[name] || name
}

function formatUptime(uptime: string): string {
  // Parse Go duration format: "1h2m3.456s", "53m56.204970015s", "2h30m", etc.
  let totalSeconds = 0

  const hourMatch = /(\d+)h/.exec(uptime)
  const minMatch = /(\d+)m/.exec(uptime)
  const secMatch = /([\d.]+)s/.exec(uptime)

  if (hourMatch) totalSeconds += parseInt(hourMatch[1], 10) * 3600
  if (minMatch) totalSeconds += parseInt(minMatch[1], 10) * 60
  if (secMatch) totalSeconds += Math.floor(parseFloat(secMatch[1]))

  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)

  if (days > 0) {
    return `${days} Tag${days > 1 ? 'e' : ''}, ${hours}h ${minutes}m`
  }
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  return `${minutes} Minuten`
}

function getVersionStatusProps(versionInfo: VersionInfo) {
  if (versionInfo.isDevelopment) {
    return {
      status: 'default' as const,
      icon: <Code className="text-dark-400" />,
      description: 'Development-Version',
    }
  }
  if (versionInfo.isStage) {
    return {
      status: 'default' as const,
      icon: <FlaskConical className="text-dark-400" />,
      description: 'Stage-Version',
    }
  }
  if (versionInfo.updateAvailable && versionInfo.latest) {
    const releaseNotesUrl = `https://green-ecolution.de/releases/${versionInfo.latest}`
    return {
      status: 'yellow' as const,
      icon: <ArrowUp className="text-yellow" />,
      description: (
        <span>
          Version {versionInfo.latest} ist verfügbar
          {' · '}
          <a
            href={releaseNotesUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-green-dark hover:underline"
          >
            Release Notes
          </a>
        </span>
      ),
    }
  }
  return {
    status: 'green-dark' as const,
    icon: undefined,
    description: 'Software ist auf dem neuesten Stand',
  }
}

function Info() {
  const { tab } = useSearch({ from: '/_protected/info' })
  const { data } = useSuspenseQuery(infoQuery())
  const { data: servicesData } = useSuspenseQuery(servicesInfoQuery())
  const { data: serverData } = useQuery(serverInfoQuery())
  const { data: statsData } = useQuery(statisticsQuery())

  const totalServices = servicesData.items.length
  const versionProps = getVersionStatusProps(data.versionInfo)
  const hasServerInfo = serverData?.hostname

  // Fallback to 'system' if tab requires server info but it's not available
  const activeTab = (tab === 'server' || tab === 'runtime') && !hasServerInfo ? 'system' : tab

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Systeminformationen
        </h1>
        <p>
          Hier findest du eine Übersicht über die aktuelle Version, den Status der verbundenen
          Services und weitere technische Details zur laufenden Instanz.
        </p>
      </article>

      <Tabs value={activeTab}>
        <TabsList>
          <TabsTrigger value="system" asChild>
            <Link to="/info" search={{ tab: 'system' }}>
              <Monitor className="size-5" />
              System
            </Link>
          </TabsTrigger>
          <TabsTrigger value="data" asChild>
            <Link to="/info" search={{ tab: 'data' }}>
              <Database className="size-5" />
              Daten
            </Link>
          </TabsTrigger>
          <TabsTrigger value="software" asChild>
            <Link to="/info" search={{ tab: 'software' }}>
              <Layers className="size-5" />
              Software
            </Link>
          </TabsTrigger>
          {hasServerInfo && (
            <TabsTrigger value="server" asChild>
              <Link to="/info" search={{ tab: 'server' }}>
                <Server className="size-5" />
                Server
              </Link>
            </TabsTrigger>
          )}
          {hasServerInfo && (
            <TabsTrigger value="runtime" asChild>
              <Link to="/info" search={{ tab: 'runtime' }}>
                <Activity className="size-5" />
                Runtime
              </Link>
            </TabsTrigger>
          )}
        </TabsList>

        <TabsContent value="system">
          <SystemTabContent
            data={data}
            servicesData={servicesData}
            serverData={serverData}
            versionProps={versionProps}
            totalServices={totalServices}
            formatUptime={formatUptime}
          />
        </TabsContent>

        <TabsContent value="software">
          <SoftwareTabContent data={data} />
        </TabsContent>

        {hasServerInfo && serverData && (
          <TabsContent value="server">
            <ServerTabContent serverData={serverData} formatUptime={formatUptime} />
          </TabsContent>
        )}

        <TabsContent value="data">
          <DataTabContent statsData={statsData} />
        </TabsContent>

        {hasServerInfo && (
          <TabsContent value="runtime">
            <RuntimeStats />
          </TabsContent>
        )}
      </Tabs>
    </div>
  )
}

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

function DataTabContent({ statsData }: DataTabContentProps) {
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
        <Card className="lg:col-span-1 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-green-dark/5 to-transparent" />
          <CardContent className="pt-6 relative">
            <div className="flex items-start justify-between">
              <div>
                <p className="text-sm font-medium text-dark-600 mb-1">Bäume gesamt</p>
                <p className="text-5xl font-bold font-lato text-green-dark tracking-tight">
                  {statsData?.treeCount?.toLocaleString('de-DE') ?? '-'}
                </p>
                <p className="text-sm text-dark-500 mt-2">
                  in {statsData?.treeClusterCount?.toLocaleString('de-DE') ?? '-'} Gruppen verwaltet
                </p>
              </div>
              <div className="p-3 bg-green-dark/10 rounded-xl">
                <TreeDeciduous className="size-8 text-green-dark" />
              </div>
            </div>
            <Link
              to="/treecluster"
              search={{ page: 1 }}
              className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
            >
              Alle Baumgruppen ansehen
              <ArrowRight className="size-4" />
            </Link>
          </CardContent>
        </Card>

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

// ============================================================================
// SYSTEM TAB
// ============================================================================

interface SystemTabContentProps {
  data: {
    versionInfo: VersionInfo
    goVersion: string
  }
  servicesData: {
    items: ServiceStatus[]
  }
  serverData:
    | {
        uptime: string
      }
    | undefined
  versionProps: {
    status: 'default' | 'yellow' | 'green-dark'
    icon: React.ReactNode
    description: React.ReactNode
  }
  totalServices: number
  formatUptime: (uptime: string) => string
}

const serviceIconMap: Record<string, React.ReactNode> = {
  database: <Database className="size-4" />,
  auth: <Shield className="size-4" />,
  mqtt: <Radio className="size-4" />,
  s3: <HardDrive className="size-4" />,
  routing: <Network className="size-4" />,
  vroom: <Zap className="size-4" />,
}

function SystemTabContent({
  data,
  servicesData,
  serverData,
  versionProps,
  totalServices,
  formatUptime,
}: SystemTabContentProps) {
  const healthyServices = servicesData.items.filter(
    (s: ServiceStatus) => s.enabled && s.healthy,
  ).length

  const version = data.versionInfo.current
  const isDev = data.versionInfo.isDevelopment || data.versionInfo.isStage
  const isLongVersion = version.length > 12

  return (
    <div className="space-y-6">
      {/* Hero section with version */}
      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-1 relative overflow-hidden">
          <div
            className={`absolute inset-0 ${
              versionProps.status === 'green-dark'
                ? 'bg-gradient-to-br from-green-dark/5 to-transparent'
                : versionProps.status === 'yellow'
                  ? 'bg-gradient-to-br from-yellow/10 to-transparent'
                  : 'bg-gradient-to-br from-dark-200/50 to-transparent'
            }`}
          />
          <CardContent className="pt-6 relative">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-dark-600 mb-1">Version</p>
                {isLongVersion ? (
                  <p
                    className="text-lg font-bold font-mono tracking-tight break-all"
                    title={version}
                  >
                    {version}
                  </p>
                ) : (
                  <p className="text-3xl font-bold font-lato tracking-tight">{version}</p>
                )}
                <div className="mt-3 flex items-center gap-2 flex-wrap">
                  {isDev && (
                    <span className="inline-flex items-center gap-1.5 px-2 py-0.5 text-xs font-medium bg-dark-200 text-dark-600 rounded">
                      <Code className="size-3" />
                      {data.versionInfo.isDevelopment ? 'Development' : 'Stage'}
                    </span>
                  )}
                  {versionProps.status === 'green-dark' && !isDev && (
                    <span className="inline-flex items-center gap-1.5 text-sm text-green-dark">
                      <CheckCircle2 className="size-4" />
                      Aktuell
                    </span>
                  )}
                  {versionProps.status === 'yellow' && (
                    <span className="inline-flex items-center gap-1.5 text-sm text-yellow-600">
                      <ArrowUp className="size-4" />
                      Update verfügbar
                    </span>
                  )}
                </div>
              </div>
              <div className="p-2.5 bg-dark-100 rounded-xl shrink-0">
                <Package className="size-6 text-dark-500" />
              </div>
            </div>
            {data.versionInfo.updateAvailable && data.versionInfo.latest && (
              <a
                href={`https://green-ecolution.de/releases/${data.versionInfo.latest}`}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
              >
                Release Notes ansehen
                <ExternalLink className="size-3" />
              </a>
            )}
          </CardContent>
        </Card>

        {/* Quick stats */}
        <Card className="lg:col-span-2">
          <CardContent className="pt-6">
            <div className="grid gap-6 sm:grid-cols-3">
              <div>
                <p className="text-sm text-dark-500 mb-1">Services</p>
                <p className="text-2xl font-bold font-lato">
                  {healthyServices}
                  <span className="text-base text-dark-400">/{totalServices}</span>
                </p>
                <p className="text-xs text-dark-400 mt-1">aktiv und gesund</p>
              </div>
              {serverData && (
                <div>
                  <p className="text-sm text-dark-500 mb-1">Uptime</p>
                  <p className="text-2xl font-bold font-lato">{formatUptime(serverData.uptime)}</p>
                  <p className="text-xs text-dark-400 mt-1">seit letztem Neustart</p>
                </div>
              )}
              <div>
                <p className="text-sm text-dark-500 mb-1">Go Runtime</p>
                <p className="text-2xl font-bold font-mono">{data.goVersion.replace('go', '')}</p>
                <p className="text-xs text-dark-400 mt-1">Backend Version</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Services grid */}
      <Card>
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Settings className="size-5" />
              Service Status
            </CardTitle>
            <span className="text-sm text-dark-500">
              {healthyServices} von {totalServices} online
            </span>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
            {servicesData.items.map((service: ServiceStatus) => {
              const isHealthy = service.enabled && service.healthy
              const isDisabled = !service.enabled

              return (
                <div
                  key={service.name}
                  className={`flex items-center gap-3 p-4 rounded-lg border transition-colors ${
                    isHealthy
                      ? 'bg-green-dark/5 border-green-dark/20'
                      : isDisabled
                        ? 'bg-dark-100/50 border-dark-200'
                        : 'bg-red/5 border-red/20'
                  }`}
                >
                  <div
                    className={`p-2 rounded-lg ${
                      isHealthy
                        ? 'bg-green-dark/10 text-green-dark'
                        : isDisabled
                          ? 'bg-dark-200 text-dark-400'
                          : 'bg-red/10 text-red'
                    }`}
                  >
                    {serviceIconMap[service.name] ?? <Settings className="size-4" />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="font-medium truncate">{getServiceDisplayName(service.name)}</p>
                      {isHealthy ? (
                        <CheckCircle2 className="size-4 text-green-dark shrink-0" />
                      ) : isDisabled ? (
                        <span className="text-xs text-dark-400">Deaktiviert</span>
                      ) : (
                        <XCircle className="size-4 text-red shrink-0" />
                      )}
                    </div>
                    <p className="text-xs text-dark-500 truncate">{service.message}</p>
                    {service.enabled &&
                      service.responseTimeMs !== undefined &&
                      service.responseTimeMs > 0 && (
                        <p className="text-xs text-dark-400 mt-0.5">
                          {service.responseTimeMs < 1
                            ? `${(service.responseTimeMs * 1000).toFixed(0)}µs`
                            : `${service.responseTimeMs.toFixed(1)}ms`}
                        </p>
                      )}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// ============================================================================
// SOFTWARE TAB
// ============================================================================

interface SoftwareTabContentProps {
  data: {
    version: string
    buildTime: string
    goVersion: string
    git: {
      branch: string
      commit: string
      repository: string
    }
    versionInfo: VersionInfo
  }
}

function SoftwareTabContent({ data }: SoftwareTabContentProps) {
  const copyToClipboard = async (text: string) => {
    await navigator.clipboard.writeText(text)
    toast.success('In Zwischenablage kopiert')
  }

  const buildDate = new Date(data.buildTime)

  return (
    <div className="space-y-6">
      {/* Version hero */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-green-dark/5 to-transparent" />
          <CardContent className="pt-6 relative">
            <div className="flex items-start justify-between mb-6">
              <div>
                <p className="text-sm font-medium text-dark-600 mb-1">Software Version</p>
                <p className="text-5xl font-bold font-lato text-green-dark tracking-tight">
                  {data.version}
                </p>
              </div>
              <div className="p-3 bg-green-dark/10 rounded-xl">
                <Tag className="size-8 text-green-dark" />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-dark-200">
              <div>
                <p className="text-xs text-dark-500 mb-1">Build-Datum</p>
                <p className="font-medium">
                  {buildDate.toLocaleDateString('de-DE', {
                    day: '2-digit',
                    month: 'long',
                    year: 'numeric',
                  })}
                </p>
              </div>
              <div>
                <p className="text-xs text-dark-500 mb-1">Build-Zeit</p>
                <p className="font-medium">
                  {buildDate.toLocaleTimeString('de-DE', {
                    hour: '2-digit',
                    minute: '2-digit',
                  })}{' '}
                  Uhr
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-cyan-500/5 to-transparent" />
          <CardContent className="pt-6 relative">
            <div className="flex items-start justify-between mb-6">
              <div>
                <p className="text-sm font-medium text-dark-600 mb-1">Go Runtime</p>
                <p className="text-5xl font-bold font-lato text-cyan-600 tracking-tight">
                  {data.goVersion.replace('go', '')}
                </p>
              </div>
              <div className="p-3 bg-cyan-500/10 rounded-xl">
                <Code className="size-8 text-cyan-600" />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-dark-200">
              <div>
                <p className="text-xs text-dark-500 mb-1">Sprache</p>
                <p className="font-medium">Go (Golang)</p>
              </div>
              <div>
                <p className="text-xs text-dark-500 mb-1">Dokumentation</p>
                <a
                  href="https://pkg.go.dev/github.com/green-ecolution/green-ecolution/backend"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 font-medium text-cyan-600 hover:underline"
                >
                  pkg.go.dev
                  <ExternalLink className="size-3" />
                </a>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Git info */}
      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="flex items-center gap-2 text-lg">
            <GitBranch className="size-5" />
            Git Repository
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <GitBranch className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">Branch</p>
              </div>
              <p className="font-medium font-mono">{data.git.branch}</p>
            </div>

            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <GitCommit className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">Commit</p>
              </div>
              <div className="flex items-center gap-2">
                <code className="font-mono text-sm truncate flex-1">{data.git.commit}</code>
                <button
                  onClick={() => void copyToClipboard(data.git.commit)}
                  className="p-1.5 hover:bg-dark-200 rounded transition-colors shrink-0 cursor-pointer"
                  title="Kopieren"
                >
                  <Copy className="size-3.5 text-dark-500 hover:text-dark-700" />
                </button>
              </div>
            </div>

            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Globe className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">Repository</p>
              </div>
              <a
                href={data.git.repository}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 font-medium text-green-dark hover:underline"
              >
                GitHub
                <ExternalLink className="size-3.5" />
              </a>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Release info */}
      {data.versionInfo.latest && (
        <Card className={data.versionInfo.updateAvailable ? 'border-yellow/30 bg-yellow/5' : ''}>
          <CardContent className="pt-6">
            <div className="flex items-start justify-between">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  {data.versionInfo.updateAvailable ? (
                    <ArrowUp className="size-5 text-yellow-600" />
                  ) : (
                    <CheckCircle2 className="size-5 text-green-dark" />
                  )}
                  <p className="font-medium">
                    {data.versionInfo.updateAvailable
                      ? 'Neue Version verfügbar'
                      : 'Software ist aktuell'}
                  </p>
                </div>
                {data.versionInfo.updateAvailable && (
                  <p className="text-sm text-dark-600 mb-4">
                    Version <span className="font-mono font-medium">{data.versionInfo.latest}</span>{' '}
                    ist verfügbar
                  </p>
                )}
                <a
                  href={`https://green-ecolution.de/releases/${data.versionInfo.latest}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
                >
                  Release Notes ansehen
                  <ExternalLink className="size-3.5" />
                </a>
              </div>
              <div className="text-right">
                <p className="text-xs text-dark-500 mb-1">Neueste Version</p>
                <p className="text-2xl font-bold font-mono">{data.versionInfo.latest}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}

// ============================================================================
// SERVER TAB
// ============================================================================

interface ServerTabContentProps {
  serverData: {
    hostname: string
    os: string
    arch: string
    port: number
    uptime: string
    ip: string
    _interface: string
    url: string
  }
  formatUptime: (uptime: string) => string
}

function ServerTabContent({ serverData, formatUptime }: ServerTabContentProps) {
  const copyToClipboard = async (text: string) => {
    await navigator.clipboard.writeText(text)
    toast.success('In Zwischenablage kopiert')
  }

  return (
    <div className="space-y-6">
      {/* Server hero */}
      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-1 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent" />
          <CardContent className="pt-6 relative">
            <div className="flex items-start justify-between">
              <div>
                <p className="text-sm font-medium text-dark-600 mb-1">Hostname</p>
                <p className="text-2xl font-bold font-lato tracking-tight break-all">
                  {serverData.hostname}
                </p>
                <p className="text-sm text-dark-500 mt-2">
                  {serverData.os} / {serverData.arch}
                </p>
              </div>
              <div className="p-3 bg-blue-500/10 rounded-xl">
                <Server className="size-8 text-blue-600" />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="lg:col-span-2">
          <CardContent className="pt-6">
            <div className="grid gap-6 md:grid-cols-3">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Clock className="size-4 text-dark-500" />
                  <p className="text-sm text-dark-500">Uptime</p>
                </div>
                <p className="text-2xl font-bold font-lato">{formatUptime(serverData.uptime)}</p>
              </div>
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Monitor className="size-4 text-dark-500" />
                  <p className="text-sm text-dark-500">Betriebssystem</p>
                </div>
                <p className="text-lg font-semibold capitalize">{serverData.os}</p>
                <p className="text-sm text-dark-400">{serverData.arch}</p>
              </div>
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Zap className="size-4 text-dark-500" />
                  <p className="text-sm text-dark-500">HTTP Port</p>
                </div>
                <p className="text-2xl font-bold font-mono">{serverData.port}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Network info */}
      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Network className="size-5" />
            Netzwerk
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Globe className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">IP-Adresse</p>
              </div>
              <div className="flex items-center gap-2">
                <code className="font-mono text-lg font-medium flex-1">{serverData.ip}</code>
                <button
                  onClick={() => void copyToClipboard(serverData.ip)}
                  className="p-1.5 hover:bg-dark-200 rounded transition-colors cursor-pointer"
                  title="Kopieren"
                >
                  <Copy className="size-4 text-dark-500 hover:text-dark-700" />
                </button>
              </div>
            </div>

            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Network className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">Interface</p>
              </div>
              <p className="font-mono text-lg font-medium">{serverData._interface}</p>
            </div>

            <div className="p-4 bg-dark-100/50 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <ExternalLink className="size-4 text-dark-500" />
                <p className="text-sm text-dark-500">URL</p>
              </div>
              <a
                href={serverData.url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 font-medium text-green-dark hover:underline break-all"
              >
                {serverData.url}
                <ExternalLink className="size-3.5 shrink-0" />
              </a>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Technical details */}
      <Card className="bg-dark-100/30">
        <CardContent className="py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-dark-200 rounded-lg">
                <Activity className="size-4 text-dark-600" />
              </div>
              <div>
                <p className="text-sm font-medium">Server-Informationen</p>
                <p className="text-xs text-dark-500">
                  {serverData.hostname} auf Port {serverData.port}
                </p>
              </div>
            </div>
            <code className="text-sm font-mono text-dark-600">
              {serverData.os}/{serverData.arch}
            </code>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
