const NAMESPACE = 'green-ecolution'
const VERSION = 1

export interface PluginContext {
  locale: 'de' | 'en'
  theme: 'light' | 'dark'
  user: { displayName: string }
  plugin: { slug: string }
}

export type Envelope<T> = {
  ns: typeof NAMESPACE
  v: typeof VERSION
  type: string
  payload: T
}

function envelope<T>(type: string, payload: T): Envelope<T> {
  return { ns: NAMESPACE, v: VERSION, type, payload }
}

function isEnvelope(data: unknown): data is Envelope<unknown> {
  if (typeof data !== 'object' || data === null) return false
  const candidate = data as Record<string, unknown>
  return candidate.ns === NAMESPACE && candidate.v === VERSION && typeof candidate.type === 'string'
}

function isResizePayload(payload: unknown): payload is { height: number } {
  return (
    typeof payload === 'object' &&
    payload !== null &&
    typeof (payload as { height?: unknown }).height === 'number'
  )
}

export interface CreatePluginHostOptions {
  origin: string
  context: PluginContext
  onResize?: (height: number) => void
}

/**
 * Both origin and source are checked on every message: origin alone would
 * accept a message forged by any other same-origin frame, and source alone
 * would accept a message replayed from a different, unrelated origin.
 */
export function createPluginHost(
  iframe: HTMLIFrameElement,
  { origin, context, onResize }: CreatePluginHostOptions,
): () => void {
  const handleMessage = (event: MessageEvent) => {
    if (event.origin !== origin) return
    if (event.source !== iframe.contentWindow) return
    if (!isEnvelope(event.data)) return

    if (event.data.type === 'ge:hello') {
      iframe.contentWindow?.postMessage(envelope('ge:init', context), origin)
      return
    }

    if (event.data.type === 'ge:resize' && isResizePayload(event.data.payload)) {
      onResize?.(event.data.payload.height)
    }
  }

  window.addEventListener('message', handleMessage)
  return () => window.removeEventListener('message', handleMessage)
}

/**
 * Runs inside the plugin's own document. The target is '*' because the
 * envelope carries no secret; the host itself only ever answers a hello that
 * came from its own iframe's contentWindow (see createPluginHost). The
 * plugin's counterpart is always window.parent, so that is checked here too
 * — nothing secret travels this way, but both sides verify each other.
 */
export interface ConnectOptions {
  /** How often the hello is repeated while no answer has arrived. 0 sends it once. */
  retryIntervalMs?: number
  /** When to give up and reject. 0 waits forever. */
  timeoutMs?: number
}

export function connectToHost({
  retryIntervalMs = 250,
  timeoutMs = 10_000,
}: ConnectOptions = {}): Promise<PluginContext> {
  return new Promise((resolve, reject) => {
    const sendHello = () => window.parent.postMessage(envelope('ge:hello', {}), '*')
    let retry: ReturnType<typeof setInterval> | undefined
    let expiry: ReturnType<typeof setTimeout> | undefined

    const handleMessage = (event: MessageEvent) => {
      if (event.source !== window.parent) return
      if (!isEnvelope(event.data) || event.data.type !== 'ge:init') return
      stop()
      resolve(event.data.payload as PluginContext)
    }

    const stop = () => {
      window.removeEventListener('message', handleMessage)
      if (retry !== undefined) clearInterval(retry)
      if (expiry !== undefined) clearTimeout(expiry)
    }

    window.addEventListener('message', handleMessage)
    sendHello()

    // The host attaches its listener from an effect and re-attaches it whenever
    // the context it passes changes, so a single hello can fall into that gap
    // and leave the plugin waiting on a blank page with nothing in the console.
    // Repeating costs one postMessage and turns the race into a short delay.
    if (retryIntervalMs > 0) retry = setInterval(sendHello, retryIntervalMs)
    if (timeoutMs > 0) {
      expiry = setTimeout(() => {
        stop()
        reject(
          new Error(
            `The Green Ecolution host did not answer the handshake within ${timeoutMs}ms. ` +
              'Is this document embedded as a plugin view?',
          ),
        )
      }, timeoutMs)
    }
  })
}

export function notifyResize(height: number): void {
  window.parent.postMessage(envelope('ge:resize', { height }), '*')
}
