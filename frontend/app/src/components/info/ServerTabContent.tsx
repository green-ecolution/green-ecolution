import { Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import { Activity, Clock, ExternalLink, Monitor, Network, Server, Zap } from 'lucide-react'
import HeroStatCard from './HeroStatCard'
import InfoTile from './InfoTile'
import { formatUptime } from './formatUptime'

interface ServerTabContentProps {
  serverData: {
    hostname: string
    os: string
    arch: string
    port: number
    uptimeSeconds: number
    _interface: string
    url: string
  }
}

const ServerTabContent = ({ serverData }: ServerTabContentProps) => {
  return (
    <div className="space-y-6">
      {/* Server hero */}
      <div className="grid gap-6 lg:grid-cols-3">
        <HeroStatCard
          className="lg:col-span-1"
          gradient="bg-gradient-to-br from-blue-500/5 to-transparent"
          icon={<Server className="size-8 text-blue-600" />}
          iconBoxClassName="p-3 bg-blue-500/10"
        >
          <div>
            <p className="text-sm font-medium text-dark-600 mb-1">Hostname</p>
            <p className="text-2xl font-bold font-lato tracking-tight break-all">
              {serverData.hostname}
            </p>
            <p className="text-sm text-dark-500 mt-2">
              {serverData.os} / {serverData.arch}
            </p>
          </div>
        </HeroStatCard>

        <Card className="lg:col-span-2">
          <CardContent className="pt-6">
            <div className="grid gap-6 md:grid-cols-3">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Clock className="size-4 text-dark-500" />
                  <p className="text-sm text-dark-500">Uptime</p>
                </div>
                <p className="text-2xl font-bold font-lato">
                  {formatUptime(serverData.uptimeSeconds)}
                </p>
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
          <div className="grid gap-4 md:grid-cols-2">
            <InfoTile icon={Network} label="Interface">
              <p className="font-mono text-lg font-medium">{serverData._interface}</p>
            </InfoTile>

            <InfoTile icon={ExternalLink} label="URL">
              <a
                href={serverData.url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 font-medium text-green-dark hover:underline break-all"
              >
                {serverData.url}
                <ExternalLink className="size-3.5 shrink-0" />
              </a>
            </InfoTile>
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

export default ServerTabContent
