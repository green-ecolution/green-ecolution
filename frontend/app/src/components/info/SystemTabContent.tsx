import { Card, CardContent, CardHeader, CardTitle, Loading } from '@green-ecolution/ui'
import {
  ArrowUp,
  CheckCircle2,
  Code,
  Database,
  ExternalLink,
  HardDrive,
  Network,
  Package,
  Puzzle,
  Radio,
  Settings,
  Shield,
  XCircle,
  Zap,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import type { ServiceStatusResponse, VersionInfoResponse } from '@green-ecolution/backend-client'
import HeroStatCard from './HeroStatCard'
import { formatUptime } from './formatUptime'

function serviceDisplayName(t: TFunction<'info'>, name: string): string {
  const map: Record<string, string> = {
    database: t('system.serviceName.database'),
    auth: t('system.serviceName.auth'),
    mqtt: t('system.serviceName.mqtt'),
    s3: t('system.serviceName.s3'),
    routing: t('system.serviceName.routing'),
    vroom: t('system.serviceName.vroom'),
    plugins: t('system.serviceName.plugins'),
  }
  return map[name] || name
}

function serviceStatusMessages(t: TFunction<'info'>): Record<string, string> {
  return {
    'service.status.disabled': t('system.serviceStatus.disabled'),
    'service.status.connected': t('system.serviceStatus.connected'),
    'service.status.no_connection': t('system.serviceStatus.noConnection'),
    'service.status.url_not_configured': t('system.serviceStatus.urlNotConfigured'),
    'service.status.not_configured': t('system.serviceStatus.notConfigured'),
    'service.status.enabled': t('system.serviceStatus.enabled'),
    'service.status.connection_error': t('system.serviceStatus.connectionError'),
    'service.status.bucket_not_found': t('system.serviceStatus.bucketNotFound'),
  }
}

function translateServiceMessage(t: TFunction<'info'>, key?: string): string {
  if (!key) return ''
  return serviceStatusMessages(t)[key] ?? key
}

type VersionStatus = 'default' | 'yellow' | 'green-dark'

function getVersionStatus(versionInfo: VersionInfoResponse): VersionStatus {
  if (versionInfo.isDevelopment || versionInfo.isStage) return 'default'
  if (versionInfo.updateAvailable && versionInfo.latest) return 'yellow'
  return 'green-dark'
}

const serviceIconMap: Record<string, React.ReactNode> = {
  database: <Database className="size-4" />,
  auth: <Shield className="size-4" />,
  mqtt: <Radio className="size-4" />,
  s3: <HardDrive className="size-4" />,
  routing: <Network className="size-4" />,
  vroom: <Zap className="size-4" />,
  plugins: <Puzzle className="size-4" />,
}

interface SystemTabContentProps {
  data: {
    versionInfo: VersionInfoResponse
    rustVersion: string
  }
  servicesData:
    | {
        items: ServiceStatusResponse[]
      }
    | undefined
  servicesLoading: boolean
  serverData:
    | {
        uptimeSeconds: number
      }
    | undefined
  totalServices: number
}

const SystemTabContent = ({
  data,
  servicesData,
  servicesLoading,
  serverData,
  totalServices,
}: SystemTabContentProps) => {
  const { t } = useTranslation('info')
  const healthyServices =
    servicesData?.items.filter((s: ServiceStatusResponse) => s.enabled && s.healthy).length ?? 0

  const versionStatus = getVersionStatus(data.versionInfo)
  const version = data.versionInfo.current
  const isDev = data.versionInfo.isDevelopment || data.versionInfo.isStage
  const isLongVersion = version.length > 12

  return (
    <div className="space-y-6">
      {/* Hero section with version */}
      <div className="grid gap-6 lg:grid-cols-3">
        <HeroStatCard
          className="lg:col-span-1"
          gradient={
            versionStatus === 'green-dark'
              ? 'bg-gradient-to-br from-green-dark/5 to-transparent'
              : versionStatus === 'yellow'
                ? 'bg-gradient-to-br from-yellow/10 to-transparent'
                : 'bg-gradient-to-br from-dark-200/50 to-transparent'
          }
          headerClassName="gap-3"
          icon={<Package className="size-6 text-dark-500" />}
          iconBoxClassName="p-2.5 bg-dark-100 shrink-0"
          footer={
            data.versionInfo.updateAvailable &&
            data.versionInfo.latest && (
              <a
                href={`https://green-ecolution.de/releases/v${data.versionInfo.latest}`}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-green-dark hover:underline"
              >
                {t('releaseNotesLink')}
                <ExternalLink className="size-3" />
              </a>
            )
          }
        >
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-dark-600 mb-1">
              {t('system.hero.versionLabel')}
            </p>
            {isLongVersion ? (
              <p className="text-lg font-bold font-mono tracking-tight break-all" title={version}>
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
              {versionStatus === 'green-dark' && !isDev && (
                <span className="inline-flex items-center gap-1.5 text-sm text-green-dark">
                  <CheckCircle2 className="size-4" />
                  {t('system.hero.upToDateBadge')}
                </span>
              )}
              {versionStatus === 'yellow' && (
                <span className="inline-flex items-center gap-1.5 text-sm text-yellow-600">
                  <ArrowUp className="size-4" />
                  {t('system.hero.updateAvailableBadge')}
                </span>
              )}
            </div>
          </div>
        </HeroStatCard>

        {/* Quick stats */}
        <Card className="lg:col-span-2">
          <CardContent className="pt-6">
            <div className="grid gap-6 sm:grid-cols-3">
              <div>
                <p className="text-sm text-dark-500 mb-1">{t('system.quick.servicesLabel')}</p>
                <p className="text-2xl font-bold font-lato">
                  {healthyServices}
                  <span className="text-base text-dark-400">/{totalServices}</span>
                </p>
                <p className="text-xs text-dark-400 mt-1">{t('system.quick.servicesHint')}</p>
              </div>
              {serverData && (
                <div>
                  <p className="text-sm text-dark-500 mb-1">{t('uptimeLabel')}</p>
                  <p className="text-2xl font-bold font-lato">
                    {formatUptime(serverData.uptimeSeconds)}
                  </p>
                  <p className="text-xs text-dark-400 mt-1">{t('system.quick.uptimeHint')}</p>
                </div>
              )}
              <div>
                <p className="text-sm text-dark-500 mb-1">{t('system.quick.backendLabel')}</p>
                <p className="text-2xl font-bold font-mono">
                  {t('system.quick.backendValue', { version: data.rustVersion })}
                </p>
                <p className="text-xs text-dark-400 mt-1">{t('system.quick.backendHint')}</p>
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
              {t('system.services.title')}
            </CardTitle>
            <span className="text-sm text-dark-500">
              {t('system.services.onlineCount', { healthy: healthyServices, total: totalServices })}
            </span>
          </div>
        </CardHeader>
        <CardContent>
          {servicesLoading ? (
            <div className="flex items-center justify-center py-8 text-dark-500">
              <Loading label={t('system.services.loadingLabel')} />
            </div>
          ) : servicesData ? (
            <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
              {servicesData.items.map((service: ServiceStatusResponse) => {
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
                        <p className="font-medium truncate">
                          {serviceDisplayName(t, service.name)}
                        </p>
                        {isHealthy ? (
                          <CheckCircle2 className="size-4 text-green-dark shrink-0" />
                        ) : isDisabled ? (
                          <span className="text-xs text-dark-400">
                            {translateServiceMessage(t, 'service.status.disabled')}
                          </span>
                        ) : (
                          <XCircle className="size-4 text-red shrink-0" />
                        )}
                      </div>
                      <p className="text-xs text-dark-500 truncate">
                        {translateServiceMessage(t, service.message ?? undefined)}
                      </p>
                      {service.enabled &&
                        service.responseTimeMs != null &&
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
          ) : (
            <div className="text-center py-8 text-dark-500">{t('system.services.emptyLabel')}</div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

export default SystemTabContent
