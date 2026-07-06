import { Card, CardContent, CardHeader, CardTitle, toast } from '@green-ecolution/ui'
import {
  ArrowUp,
  CheckCircle2,
  Code,
  Copy,
  ExternalLink,
  FlaskConical,
  GitBranch,
  GitCommit,
  Globe,
  Package,
  Tag,
} from 'lucide-react'
import type { VersionInfoResponse } from '@green-ecolution/backend-client'
import HeroStatCard from './HeroStatCard'
import InfoTile from './InfoTile'

type BuildEnv = 'development' | 'stage' | 'release'

function getBuildEnv(versionInfo: VersionInfoResponse): BuildEnv {
  if (versionInfo.isDevelopment) return 'development'
  if (versionInfo.isStage) return 'stage'
  return 'release'
}

interface EnvVisual {
  label: string
  heroGradient: string
  heroIconBg: string
  heroIconColor: string
  versionColor: string
  pillBg: string
  pillText: string
  pillIcon: React.ReactNode
}

const envVisuals: Record<BuildEnv, EnvVisual> = {
  release: {
    label: 'Release',
    heroGradient: 'from-green-dark/5 to-transparent',
    heroIconBg: 'bg-green-dark/10',
    heroIconColor: 'text-green-dark',
    versionColor: 'text-green-dark',
    pillBg: 'bg-green-dark/10',
    pillText: 'text-green-dark',
    pillIcon: <CheckCircle2 className="size-3.5" />,
  },
  stage: {
    label: 'Staging',
    heroGradient: 'from-blue-500/8 to-transparent',
    heroIconBg: 'bg-blue-500/10',
    heroIconColor: 'text-blue-600',
    versionColor: 'text-blue-600',
    pillBg: 'bg-blue-500/10',
    pillText: 'text-blue-700',
    pillIcon: <FlaskConical className="size-3.5" />,
  },
  development: {
    label: 'Entwicklung',
    heroGradient: 'from-amber-500/10 to-transparent',
    heroIconBg: 'bg-amber-500/10',
    heroIconColor: 'text-amber-700',
    versionColor: 'text-amber-700',
    pillBg: 'bg-amber-500/10',
    pillText: 'text-amber-800',
    pillIcon: <Code className="size-3.5" />,
  },
}

interface SoftwareTabContentProps {
  data: {
    version: string
    buildTime: string
    rustVersion: string
    rustChannel: string
    rustEdition: string
    git: {
      branch: string
      commit: string
      repository: string
    }
    versionInfo: VersionInfoResponse
  }
}

const SoftwareTabContent = ({ data }: SoftwareTabContentProps) => {
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      toast.success('In Zwischenablage kopiert')
    } catch {
      toast.error('Kopieren fehlgeschlagen')
    }
  }

  const buildDate = new Date(data.buildTime)
  const env = getBuildEnv(data.versionInfo)
  const visual = envVisuals[env]
  const buildMode = data.versionInfo.isDevelopment ? 'Debug' : 'Release'
  const isLongVersion = data.version.length > 12

  return (
    <div className="space-y-6">
      {/* Version hero */}
      <div className="grid gap-6 lg:grid-cols-2">
        <HeroStatCard
          gradient={`bg-gradient-to-br ${visual.heroGradient}`}
          headerClassName="mb-4 min-h-[7.5rem]"
          icon={<Tag className={`size-8 ${visual.heroIconColor}`} />}
          iconBoxClassName={`p-3 ${visual.heroIconBg}`}
          footer={
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
          }
        >
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-dark-600 mb-1">Software Version</p>
            {isLongVersion ? (
              <p
                className={`text-3xl font-bold font-mono tracking-tight break-all ${visual.versionColor}`}
                title={data.version}
              >
                {data.version}
              </p>
            ) : (
              <p className={`text-5xl font-bold font-lato tracking-tight ${visual.versionColor}`}>
                {data.version}
              </p>
            )}
            <span
              className={`mt-3 inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-medium rounded-full ${visual.pillBg} ${visual.pillText}`}
            >
              {visual.pillIcon}
              {visual.label}
            </span>
          </div>
        </HeroStatCard>

        <HeroStatCard
          gradient="bg-gradient-to-br from-orange-500/5 to-transparent"
          headerClassName="mb-4 min-h-[7.5rem]"
          icon={<Code className="size-8 text-orange-600" />}
          iconBoxClassName="p-3 bg-orange-500/10"
          footer={
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-dark-200">
              <div>
                <p className="text-xs text-dark-500 mb-1">Build-Modus</p>
                <p className="font-medium">{buildMode}</p>
              </div>
              <div>
                <p className="text-xs text-dark-500 mb-1">Edition</p>
                <p className="font-medium">{data.rustEdition}</p>
              </div>
            </div>
          }
        >
          <div>
            <p className="text-sm font-medium text-dark-600 mb-1">Rust Runtime</p>
            <p className="text-5xl font-bold font-lato text-orange-600 tracking-tight">
              {data.rustVersion}
            </p>
            <span className="mt-3 inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-medium rounded-full bg-orange-500/10 text-orange-700">
              <Package className="size-3.5" />
              {data.rustChannel}
            </span>
          </div>
        </HeroStatCard>
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
            <InfoTile icon={GitBranch} label="Branch">
              <p className="font-medium font-mono">{data.git.branch}</p>
            </InfoTile>

            <InfoTile icon={GitCommit} label="Commit">
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
            </InfoTile>

            <InfoTile icon={Globe} label="Repository">
              <a
                href={data.git.repository}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 font-medium text-green-dark hover:underline"
              >
                GitHub
                <ExternalLink className="size-3.5" />
              </a>
            </InfoTile>
          </div>
        </CardContent>
      </Card>

      {/* Build status */}
      <BuildStatusCard env={env} visual={visual} versionInfo={data.versionInfo} />
    </div>
  )
}

interface BuildStatusCardProps {
  env: BuildEnv
  visual: EnvVisual
  versionInfo: VersionInfoResponse
}

function BuildStatusCard({ env, visual, versionInfo }: BuildStatusCardProps) {
  if (env === 'development') {
    return (
      <Card className="border-amber-500/30 bg-amber-500/5">
        <CardContent className="pt-6">
          <div className="flex items-start gap-4">
            <div className={`p-2.5 rounded-xl ${visual.heroIconBg} shrink-0`}>
              <Code className={`size-5 ${visual.heroIconColor}`} />
            </div>
            <div>
              <p className="font-medium mb-1">Lokaler Entwicklungs-Build</p>
              <p className="text-sm text-dark-600">
                Diese Version wurde lokal kompiliert. Der Update-Check ist deaktiviert; für
                produktive Versionen siehe{' '}
                <a
                  href="https://green-ecolution.de"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-amber-700 hover:underline font-medium"
                >
                  green-ecolution.de
                </a>
                .
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (env === 'stage') {
    return (
      <Card className="border-blue-500/30 bg-blue-500/5">
        <CardContent className="pt-6">
          <div className="flex items-start gap-4">
            <div className={`p-2.5 rounded-xl ${visual.heroIconBg} shrink-0`}>
              <FlaskConical className={`size-5 ${visual.heroIconColor}`} />
            </div>
            <div>
              <p className="font-medium mb-1">Staging-Umgebung</p>
              <p className="text-sm text-dark-600">
                Diese Version wird vor dem Release getestet. Der Update-Check ist deaktiviert; die
                produktive Version findest du unter{' '}
                <a
                  href="https://green-ecolution.de"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-700 hover:underline font-medium"
                >
                  green-ecolution.de
                </a>
                .
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (!versionInfo.latest) return null

  return (
    <Card className={versionInfo.updateAvailable ? 'border-yellow/30 bg-yellow/5' : ''}>
      <CardContent className="pt-6">
        <div className="flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2 mb-2">
              {versionInfo.updateAvailable ? (
                <ArrowUp className="size-5 text-yellow-600" />
              ) : (
                <CheckCircle2 className="size-5 text-green-dark" />
              )}
              <p className="font-medium">
                {versionInfo.updateAvailable ? 'Neue Version verfügbar' : 'Software ist aktuell'}
              </p>
            </div>
            {versionInfo.updateAvailable && (
              <p className="text-sm text-dark-600 mb-4">
                Version <span className="font-mono font-medium">{versionInfo.latest}</span> ist
                verfügbar
              </p>
            )}
            <a
              href={`https://green-ecolution.de/releases/v${versionInfo.latest}`}
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
            <p className="text-2xl font-bold font-mono">{versionInfo.latest}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

export default SoftwareTabContent
