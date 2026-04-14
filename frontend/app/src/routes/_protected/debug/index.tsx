import KV from '@/components/debug/KV'
import { boolBadge } from '@/components/debug/badgeHelpers'
import useStore from '@/store/store'
import { useAuthStore, useMapStore, useUserStore } from '@/store/store'
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  DetailedList,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@green-ecolution/ui'
import { createFileRoute, Link } from '@tanstack/react-router'
import { Copy, MapPin, QrCode } from 'lucide-react'
import { useState } from 'react'
import { toast } from '@green-ecolution/ui'
import { Eye } from 'lucide-react'

export const Route = createFileRoute('/_protected/debug/')({
  component: Debug,
})

const truncate = (s: string, max = 24) => (s.length > max ? `${s.slice(0, max)}…` : s)

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('In Zwischenablage kopiert')
  } catch {
    toast.error('Kopieren fehlgeschlagen')
  }
}

function Debug() {
  const authStore = useAuthStore()
  const mapStore = useMapStore()
  const userStore = useUserStore()
  const formDrafts = useStore((s) => s.formDrafts)

  const [env] = useState(() => ({
    isSecureContext: typeof window !== 'undefined' && window.isSecureContext,
    userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
  }))

  const draftEntries = Object.entries(formDrafts)
  const [draftPreview, setDraftPreview] = useState<{ key: string; data: unknown } | null>(null)

  return (
    <div className="container mt-6 pb-10">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">Debugging</h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Entwickler-Dashboard mit Übersicht über Stores, Umgebung und App-Konfiguration. Nur in
          Entwicklungs-Builds verfügbar.
        </p>
      </article>

      <div className="mt-6 flex flex-wrap gap-2">
        <Button asChild variant="outline" size="sm">
          <Link to="/debug/qr-scanner">
            <QrCode />
            QR-Scanner öffnen
          </Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <Link to="/debug/geolocation">
            <MapPin />
            GPS-Ortung öffnen
          </Link>
        </Button>
      </div>

      <div className="mt-6 flex flex-col gap-4">
        {/* Umgebung + Auth nebeneinander */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card variant="outlined">
            <CardHeader>
              <CardTitle className="text-base">Umgebung &amp; App</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-2 text-sm">
              <KV label="Version">
                <span className="font-mono">{__APP_VERSION__}</span>
              </KV>
              <KV label="Build">
                <span className="font-mono">{__APP_BUILD_TIME__}</span>
              </KV>
              <KV label="Stadt">
                <span className="font-mono">{__APP_CITY__}</span>
              </KV>
              <KV label="Backend-URL">
                <span className="font-mono text-xs">{import.meta.env.VITE_BACKEND_BASEURL}</span>
              </KV>
              <KV label="Modus">
                <Badge variant={import.meta.env.DEV ? 'success' : 'warning'}>
                  {import.meta.env.MODE}
                </Badge>
              </KV>
              <KV label="Base-URL">
                <span className="font-mono text-xs">{import.meta.env.BASE_URL}</span>
              </KV>
              <KV label="Secure Context">{boolBadge(env.isSecureContext)}</KV>
              <KV label="User-Agent">
                <span className="font-mono text-xs break-all text-muted-foreground">
                  {env.userAgent}
                </span>
              </KV>
            </CardContent>
          </Card>

          <Card variant="outlined">
            <CardHeader>
              <CardTitle className="text-base">Authentifizierung</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-2 text-sm">
              <KV label="Angemeldet">{boolBadge(authStore.isAuthenticated)}</KV>
              <KV label="Token vorhanden">{boolBadge(!!authStore.token)}</KV>
              <KV label="Ablauf">
                <span className="font-mono text-xs">
                  {authStore.token?.expiry
                    ? new Date(authStore.token.expiry).toLocaleString('de-DE')
                    : '—'}
                </span>
              </KV>
              <KV label="Läuft bald ab">
                {authStore.token ? (
                  boolBadge(authStore.isTokenExpiringSoon())
                ) : (
                  <span className="text-muted-foreground">—</span>
                )}
              </KV>
              <KV label="Access-Token">
                <span className="inline-flex items-center gap-1.5">
                  <span className="font-mono text-xs break-all text-muted-foreground">
                    {authStore.token?.accessToken ? truncate(authStore.token.accessToken) : '—'}
                  </span>
                  {authStore.token?.accessToken && (
                    <button
                      onClick={() => void copyToClipboard(authStore.token!.accessToken)}
                      className="p-1 hover:bg-dark-200 rounded transition-colors shrink-0 cursor-pointer"
                      title="Access-Token kopieren"
                    >
                      <Copy className="size-3.5 text-dark-500 hover:text-dark-700" />
                    </button>
                  )}
                </span>
              </KV>
              <KV label="Refresh-Token">
                <span className="inline-flex items-center gap-1.5">
                  <span className="font-mono text-xs break-all text-muted-foreground">
                    {authStore.token?.refreshToken ? truncate(authStore.token.refreshToken) : '—'}
                  </span>
                  {authStore.token?.refreshToken && (
                    <button
                      onClick={() => void copyToClipboard(authStore.token!.refreshToken)}
                      className="p-1 hover:bg-dark-200 rounded transition-colors shrink-0 cursor-pointer"
                      title="Refresh-Token kopieren"
                    >
                      <Copy className="size-3.5 text-dark-500 hover:text-dark-700" />
                    </button>
                  )}
                </span>
              </KV>
            </CardContent>
          </Card>
        </div>

        {/* Benutzer */}
        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Benutzer</CardTitle>
          </CardHeader>
          <CardContent>
            <DetailedList
              columns={2}
              details={[
                { label: 'Benutzername', value: userStore.username || '—' },
                { label: 'E-Mail', value: userStore.email || '—' },
                { label: 'Vorname', value: userStore.firstName || '—' },
                { label: 'Nachname', value: userStore.lastName || '—' },
                {
                  label: 'Status',
                  value: <Badge variant="muted">{userStore.userStatus}</Badge>,
                },
                {
                  label: 'Rollen',
                  value:
                    userStore.userRoles.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {userStore.userRoles.map((role) => (
                          <Badge key={role} variant="outline">
                            {role}
                          </Badge>
                        ))}
                      </div>
                    ) : (
                      '—'
                    ),
                },
                {
                  label: 'Führerscheine',
                  value:
                    userStore.drivingLicenses.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {userStore.drivingLicenses.map((license) => (
                          <Badge key={license} variant="outline">
                            {license}
                          </Badge>
                        ))}
                      </div>
                    ) : (
                      '—'
                    ),
                },
              ]}
            />
          </CardContent>
        </Card>

        {/* Karten-Store */}
        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Karten-Store</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <KV label="Mittelpunkt">
              <span className="inline-flex items-center gap-1.5">
                <span className="font-mono text-xs">
                  {mapStore.mapCenter[0].toFixed(6)}, {mapStore.mapCenter[1].toFixed(6)}
                </span>
                <button
                  onClick={() =>
                    void copyToClipboard(
                      `${mapStore.mapCenter[0].toFixed(6)}, ${mapStore.mapCenter[1].toFixed(6)}`,
                    )
                  }
                  className="p-1 hover:bg-dark-200 rounded transition-colors shrink-0 cursor-pointer"
                  title="Mittelpunkt kopieren"
                >
                  <Copy className="size-3.5 text-dark-500 hover:text-dark-700" />
                </button>
              </span>
            </KV>
            <KV label="Zoom">
              <span className="font-mono">{mapStore.mapZoom}</span>
            </KV>
            <KV label="Min-Zoom">
              <span className="font-mono">{mapStore.mapMinZoom}</span>
            </KV>
            <KV label="Max-Zoom">
              <span className="font-mono">{mapStore.mapMaxZoom}</span>
            </KV>
            <KV label="Auswahl-Modal">{boolBadge(mapStore.showSelectModal)}</KV>
          </CardContent>
        </Card>

        {/* Formular-Entwürfe */}
        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">
              Formular-Entwürfe{' '}
              <span className="text-muted-foreground font-normal">({draftEntries.length})</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="pt-0 px-0">
            {draftEntries.length === 0 ? (
              <p className="px-6 py-4 text-sm text-muted-foreground">Keine aktiven Entwürfe.</p>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Schlüssel</TableHead>
                    <TableHead className="w-28">Änderungen</TableHead>
                    <TableHead>Daten-Vorschau</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {draftEntries.map(([key, draft]) => (
                    <TableRow key={key}>
                      <TableCell className="font-mono text-xs">{key}</TableCell>
                      <TableCell>
                        {draft?.hasChanges ? (
                          <Badge variant="warning">Ja</Badge>
                        ) : (
                          <Badge variant="muted">Nein</Badge>
                        )}
                      </TableCell>
                      <TableCell className="font-mono text-xs text-muted-foreground">
                        {draft?.data ? (
                          <button
                            onClick={() => setDraftPreview({ key, data: draft.data })}
                            className="inline-flex items-center gap-1.5 text-left hover:text-foreground transition-colors cursor-pointer"
                            title="Daten anzeigen"
                          >
                            <span className="truncate max-w-xs">
                              {truncate(JSON.stringify(draft.data), 80)}
                            </span>
                            <Eye className="size-3.5 shrink-0" />
                          </button>
                        ) : (
                          '—'
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={!!draftPreview} onOpenChange={() => setDraftPreview(null)}>
        <DialogContent className="max-w-2xl max-h-[80vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>
              Entwurf: <span className="font-mono">{draftPreview?.key}</span>
            </DialogTitle>
          </DialogHeader>
          <pre className="overflow-auto rounded-lg bg-dark-50 p-4 text-xs font-mono">
            {draftPreview?.data ? JSON.stringify(draftPreview.data, null, 2) : '—'}
          </pre>
        </DialogContent>
      </Dialog>
    </div>
  )
}
