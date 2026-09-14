export interface PluginIdentity {
  slug: string
  name: string
  enabled: boolean
  permissions: string[]
  organization_id: string
}

export interface SessionState {
  connected: boolean
  identity: PluginIdentity | null
}

export interface ActionResult {
  status: number
  body: unknown
}

export type ActionName = 'import' | 'modify' | 'delete' | 'refs'

const result = async (response: Response): Promise<ActionResult> => ({
  status: response.status,
  body: await response.json().catch(() => null),
})

export const getSession = async (): Promise<SessionState> => {
  const response = await fetch('/api/session')
  return (await response.json()) as SessionState
}

export const connect = async (key: string): Promise<ActionResult> =>
  result(
    await fetch('/api/session', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ key }),
    }),
  )

export const disconnect = async (): Promise<ActionResult> =>
  result(await fetch('/api/session', { method: 'DELETE' }))

export const runAction = async (name: ActionName): Promise<ActionResult> =>
  result(await fetch(`/api/actions/${name}`, { method: name === 'refs' ? 'GET' : 'POST' }))
