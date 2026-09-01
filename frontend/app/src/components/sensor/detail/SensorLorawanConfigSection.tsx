import { useMemo, useState } from 'react'
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
  Input,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  cn,
  toast,
} from '@green-ecolution/ui'
import {
  ChevronDown,
  ChevronUp,
  Copy,
  Eye,
  EyeOff,
  Search,
  ShieldAlert,
  SlidersHorizontal,
} from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import { SECRET_MASK, isSensitiveConfigKey, redactConfig } from './secrets'
import { useSecretReveal } from '@/hooks/useSecretReveal'

interface SensorLorawanConfigSectionProps {
  sensor: Sensor
}

const stringifyValue = (v: unknown): string => {
  if (v === null || v === undefined) return ''
  if (typeof v === 'string') return v
  if (typeof v === 'number' || typeof v === 'boolean' || typeof v === 'bigint') {
    return String(v)
  }
  return JSON.stringify(v)
}

const AUTO_HIDE_SECONDS = 10

interface ConfigRowProps {
  configKey: string
  value: unknown
}

const ConfigRow = ({ configKey, value }: ConfigRowProps) => {
  const { t } = useTranslation('sensor')
  const sensitive = isSensitiveConfigKey(configKey)
  const { revealed, toggle } = useSecretReveal(AUTO_HIDE_SECONDS)

  const isEmpty = value === null || value === undefined || value === ''
  const stringified = stringifyValue(value)
  const displayed = sensitive && !revealed ? SECRET_MASK : stringified

  return (
    <TableRow>
      <TableCell className="font-mono text-xs font-semibold align-top">
        <span className="flex items-center gap-1.5">
          {configKey}
          {sensitive && (
            <ShieldAlert
              className="size-3 text-yellow"
              aria-label={t('secretReveal.sensitiveDataAriaLabel')}
            />
          )}
        </span>
      </TableCell>
      <TableCell className="font-mono text-xs break-all whitespace-pre-wrap text-foreground">
        <div className="flex items-start gap-2">
          <span
            className={cn(
              'flex-1',
              sensitive && !revealed && 'tracking-[0.2em] text-dark-600 select-none',
            )}
          >
            {isEmpty ? (
              <span className="text-muted-foreground italic">{t('lorawanConfig.emptyValue')}</span>
            ) : (
              displayed
            )}
          </span>
          {sensitive && !isEmpty && (
            <button
              type="button"
              onClick={toggle}
              aria-label={
                revealed
                  ? t('secretReveal.hideAriaLabel', { label: configKey })
                  : t('secretReveal.showAriaLabel', { label: configKey })
              }
              aria-pressed={revealed}
              className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-dark-100 transition shrink-0 cursor-pointer"
            >
              {revealed ? <EyeOff className="size-3.5" /> : <Eye className="size-3.5" />}
            </button>
          )}
        </div>
      </TableCell>
    </TableRow>
  )
}

const SensorLorawanConfigSection = ({ sensor }: SensorLorawanConfigSectionProps) => {
  const { t } = useTranslation('sensor')
  const config = sensor.lorawan?.config as Record<string, unknown> | undefined
  const [expanded, setExpanded] = useState(false)
  const [query, setQuery] = useState('')

  const entries = useMemo(() => {
    if (!config) return []
    return Object.entries(config).sort(([a], [b]) => a.localeCompare(b))
  }, [config])

  const hasSecrets = useMemo(() => entries.some(([k]) => isSensitiveConfigKey(k)), [entries])

  const filtered = useMemo(() => {
    if (!query.trim()) return entries
    const q = query.toLowerCase()
    return entries.filter(
      ([k, v]) =>
        k.toLowerCase().includes(q) ||
        // Don't expose sensitive values via filter matches.
        (!isSensitiveConfigKey(k) && stringifyValue(v).toLowerCase().includes(q)),
    )
  }, [entries, query])

  if (!config || entries.length === 0) return null

  const handleCopyAll = () => {
    const text = JSON.stringify(redactConfig(config), null, 2)
    navigator.clipboard.writeText(text).then(
      () =>
        toast.success(
          hasSecrets
            ? t('lorawanConfig.copiedWithSecretsToast')
            : t('lorawanConfig.copiedToast'),
        ),
      () => toast.error(t('clipboard.copyFailed')),
    )
  }

  return (
    <Card variant="outlined">
      <CardHeader>
        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
          className="flex w-full items-center justify-between gap-3 text-left cursor-pointer group"
        >
          <div className="flex items-center gap-3">
            <div className="grid place-items-center size-9 rounded-lg bg-dark text-green-light">
              <SlidersHorizontal className="size-5" />
            </div>
            <div className="flex flex-col">
              <CardTitle>{t('lorawanConfig.title')}</CardTitle>
              <span className="text-xs text-muted-foreground mt-1">
                {t('lorawanConfig.keyCountLabel', { count: entries.length })} ·{' '}
                {expanded ? t('lorawanConfig.collapseLabel') : t('lorawanConfig.expandLabel')}
              </span>
            </div>
          </div>
          {expanded ? (
            <ChevronUp className="size-5 text-muted-foreground transition group-hover:text-foreground shrink-0" />
          ) : (
            <ChevronDown className="size-5 text-muted-foreground transition group-hover:text-foreground shrink-0" />
          )}
        </button>
      </CardHeader>
      {expanded && (
        <CardContent className="flex flex-col gap-4">
          {hasSecrets && (
            <Alert variant="warning" className="w-full">
              <div className="flex gap-3">
                <AlertIcon variant="warning" />
                <AlertContent>
                  <AlertTitle>{t('lorawanConfig.secretsWarningTitle')}</AlertTitle>
                  <AlertDescription>{t('lorawanConfig.secretsWarningDescription')}</AlertDescription>
                </AlertContent>
              </div>
            </Alert>
          )}

          <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t('lorawanConfig.searchPlaceholder')}
                className="pl-9"
                aria-label={t('lorawanConfig.searchAriaLabel')}
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopyAll}
              className="gap-2 [&_svg]:size-4"
            >
              <Copy />
              {t('lorawanConfig.copyAllButton')}
            </Button>
          </div>

          <div className="overflow-hidden rounded-xl border border-dark-100">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-44 font-mono text-xs uppercase tracking-wider">
                    {t('lorawanConfig.keyColumnHeader')}
                  </TableHead>
                  <TableHead className="font-mono text-xs uppercase tracking-wider">
                    {t('lorawanConfig.valueColumnHeader')}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filtered.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={2}
                      className="text-center text-sm text-muted-foreground py-8"
                    >
                      {t('lorawanConfig.noResults', { query })}
                    </TableCell>
                  </TableRow>
                ) : (
                  filtered.map(([key, value]) => (
                    <ConfigRow key={key} configKey={key} value={value} />
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      )}
    </Card>
  )
}

export default SensorLorawanConfigSection
