import { Badge } from '@green-ecolution/ui'

export const boolBadge = (value: boolean) =>
  value ? <Badge variant="success">true</Badge> : <Badge variant="error">false</Badge>

export type PermissionLabel = PermissionState | 'unknown' | 'unavailable'

export const permissionBadge = (state: PermissionLabel) => {
  switch (state) {
    case 'granted':
      return <Badge variant="success">{state}</Badge>
    case 'denied':
      return <Badge variant="error">{state}</Badge>
    case 'prompt':
      return <Badge variant="warning">{state}</Badge>
    default:
      return <Badge variant="muted">{state}</Badge>
  }
}

export const formatTime = (ts: number) => {
  const d = new Date(ts)
  return (
    d.toLocaleTimeString('de-DE', { hour12: false }) +
    '.' +
    String(d.getMilliseconds()).padStart(3, '0')
  )
}
