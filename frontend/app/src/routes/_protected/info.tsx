import { infoQuery } from '@/api/queries'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  ListCard,
  ListCardContent,
  ListCardStatus,
  Loading,
  StatusCard,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@green-ecolution/ui'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { ArrowUp, Code, FlaskConical, Layers, Monitor, Server, Settings } from 'lucide-react'
import type { ServiceStatus, VersionInfo } from '@green-ecolution/backend-client'

export const Route = createFileRoute('/_protected/info')({
  component: Info,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Lade Systeminformationen" />
  ),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(infoQuery()),
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
  const { data } = useSuspenseQuery(infoQuery())

  const enabledServices = data.services.items.filter((s: ServiceStatus) => s.enabled)
  const totalServices = data.services.items.length
  const versionProps = getVersionStatusProps(data.versionInfo)
  const hasServerInfo = data.server?.hostname

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

      <Tabs defaultValue="system">
        <TabsList>
          <TabsTrigger value="system">
            <Monitor className="size-5" />
            System
          </TabsTrigger>
          <TabsTrigger value="software">
            <Layers className="size-5" />
            Software
          </TabsTrigger>
          {hasServerInfo && (
            <TabsTrigger value="server">
              <Server className="size-5" />
              Server
            </TabsTrigger>
          )}
        </TabsList>

        <TabsContent value="system">
          <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4 mb-8">
            <StatusCard
              status={versionProps.status}
              indicator="dot"
              icon={versionProps.icon}
              label="Version"
              value={data.versionInfo.current}
              description={versionProps.description}
            />
            <StatusCard
              status="green-dark"
              indicator="dot"
              label="Services"
              value={`${enabledServices.length}/${totalServices}`}
              description={`${enabledServices.length} von ${totalServices} Services aktiv`}
            />
            {hasServerInfo && (
              <StatusCard
                label="Uptime"
                value={formatUptime(data.server.uptime)}
                description="Zeit seit dem letzten Neustart"
              />
            )}
            <StatusCard label="Go Version" value={data.goVersion} description="Backend Runtime" />
          </div>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Settings className="size-5" />
                Services
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-3 md:grid-cols-2">
                {data.services.items.map((service: ServiceStatus) => (
                  <ListCard key={service.name} size="compact" hoverable={false}>
                    <ListCardStatus status={service.enabled ? 'green-dark' : 'dark'}>
                      {getServiceDisplayName(service.name)}
                    </ListCardStatus>
                    <ListCardContent>
                      <span className="text-sm text-dark-600">{service.message}</span>
                    </ListCardContent>
                  </ListCard>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="software">
          <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-3 mb-8">
            <StatusCard
              label="Version"
              value={data.version}
              description="Aktuelle Software-Version"
            />
            <StatusCard
              label="Build-Zeit"
              value={new Date(data.buildTime).toLocaleDateString('de-DE', {
                day: '2-digit',
                month: '2-digit',
                year: 'numeric',
                hour: '2-digit',
                minute: '2-digit',
              })}
              description="Zeitpunkt des Builds"
            />
            <StatusCard
              label="Go Version"
              value={data.goVersion}
              description="Backend Runtime Version"
            />
          </div>

          <div className="grid gap-5 md:grid-cols-2 mb-8">
            <Card>
              <CardHeader>
                <CardTitle>Git Informationen</CardTitle>
              </CardHeader>
              <CardContent>
                <dl className="grid gap-4">
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">Branch</dt>
                    <dd className="font-medium">{data.git.branch}</dd>
                  </div>
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">Commit</dt>
                    <dd className="font-medium font-mono text-sm">{data.git.commit}</dd>
                  </div>
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">Repository</dt>
                    <dd className="font-medium">
                      <a
                        href={data.git.repository}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-green-dark hover:underline"
                      >
                        GitHub
                      </a>
                    </dd>
                  </div>
                </dl>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Release</CardTitle>
              </CardHeader>
              <CardContent>
                <dl className="grid gap-4">
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">Aktuelle Version</dt>
                    <dd className="font-medium">{data.versionInfo.current}</dd>
                  </div>
                  {data.versionInfo.latest && (
                    <div>
                      <dt className="text-sm text-dark-600 mb-1">Neueste Version</dt>
                      <dd className="font-medium">
                        {data.versionInfo.latest}
                        {data.versionInfo.updateAvailable && (
                          <span className="ml-2 text-yellow text-sm">(Update verfügbar)</span>
                        )}
                      </dd>
                    </div>
                  )}
                  {data.versionInfo.latest && (
                    <div>
                      <dt className="text-sm text-dark-600 mb-1">Release Notes</dt>
                      <dd className="font-medium">
                        <a
                          href={`https://green-ecolution.de/releases/${data.versionInfo.latest}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-green-dark hover:underline"
                        >
                          Auf der Website ansehen
                        </a>
                      </dd>
                    </div>
                  )}
                </dl>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {hasServerInfo && (
          <TabsContent value="server">
            <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4 mb-8">
              <StatusCard
                label="Hostname"
                value={data.server.hostname}
                description="Server-Hostname"
              />
              <StatusCard
                label="Betriebssystem"
                value={`${data.server.os}/${data.server.arch}`}
                description="OS und Architektur"
              />
              <StatusCard
                label="Port"
                value={data.server.port.toString()}
                description="HTTP Server Port"
              />
              <StatusCard
                label="Uptime"
                value={formatUptime(data.server.uptime)}
                description="Zeit seit dem letzten Neustart"
              />
            </div>

            <Card>
              <CardHeader>
                <CardTitle>Netzwerk</CardTitle>
              </CardHeader>
              <CardContent>
                <dl className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">IP-Adresse</dt>
                    <dd className="font-medium font-mono">{data.server.ip}</dd>
                  </div>
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">Interface</dt>
                    <dd className="font-medium font-mono">{data.server._interface}</dd>
                  </div>
                  <div>
                    <dt className="text-sm text-dark-600 mb-1">URL</dt>
                    <dd className="font-medium">
                      <a
                        href={data.server.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-green-dark hover:underline"
                      >
                        {data.server.url}
                      </a>
                    </dd>
                  </div>
                </dl>
              </CardContent>
            </Card>
          </TabsContent>
        )}
      </Tabs>
    </div>
  )
}
