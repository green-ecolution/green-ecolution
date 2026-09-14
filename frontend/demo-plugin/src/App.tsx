import { useCallback, useEffect, useState } from 'react'
import { notifyResize, usePluginContext } from '@green-ecolution/plugin-interface'
import {
  connect,
  disconnect,
  getSession,
  runAction,
  type ActionName,
  type ActionResult,
  type SessionState,
} from './api'
import { strings } from './strings'

const ACTIONS: ActionName[] = ['import', 'modify', 'delete', 'refs']

export default function App() {
  const { locale, theme, user, plugin } = usePluginContext()
  const t = strings[locale] ?? strings.de

  const [session, setSession] = useState<SessionState>({ connected: false, identity: null })
  const [key, setKey] = useState('')
  const [answer, setAnswer] = useState<ActionResult | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    void getSession().then(setSession)
  }, [])

  // The static placeholder in index.html covers the time before the handshake
  // completes, because PluginProvider renders nothing at all until then.
  useEffect(() => {
    document.getElementById('not-embedded')?.remove()
  }, [])

  useEffect(() => {
    const report = () => notifyResize(document.documentElement.scrollHeight)
    report()
    const observer = new ResizeObserver(report)
    observer.observe(document.documentElement)
    return () => observer.disconnect()
  }, [])

  const submitKey = useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault()
      setBusy(true)
      setAnswer(await connect(key))
      setSession(await getSession())
      setKey('')
      setBusy(false)
    },
    [key],
  )

  const forgetKey = useCallback(async () => {
    setBusy(true)
    setAnswer(await disconnect())
    setSession(await getSession())
    setBusy(false)
  }, [])

  const trigger = useCallback(async (name: ActionName) => {
    setBusy(true)
    setAnswer(await runAction(name))
    setBusy(false)
  }, [])

  return (
    <main className="page">
      <h1>{t.title}</h1>
      <p>{t.intro}</p>

      <section>
        <h2>{t.greeting(user.displayName)}</h2>
        <dl>
          <dt>{t.slug}</dt>
          <dd>{plugin.slug}</dd>
          <dt>{t.locale}</dt>
          <dd>{locale}</dd>
          <dt>{t.theme}</dt>
          <dd>{theme}</dd>
        </dl>
      </section>

      <section>
        {session.connected && session.identity ? (
          <>
            <p>{t.connected(session.identity.slug)}</p>
            <button type="button" onClick={() => void forgetKey()} disabled={busy}>
              {t.disconnect}
            </button>
            <h2>{t.actions}</h2>
            <div className="actions">
              {ACTIONS.map((name) => (
                <button key={name} type="button" onClick={() => void trigger(name)} disabled={busy}>
                  {t[name]}
                </button>
              ))}
            </div>
          </>
        ) : (
          <form onSubmit={(event) => void submitKey(event)}>
            <label htmlFor="api-key">{t.keyLabel}</label>
            <input
              id="api-key"
              type="password"
              value={key}
              onChange={(event) => setKey(event.target.value)}
              placeholder="gep_..."
              autoComplete="off"
            />
            <p className="hint">{t.keyHint}</p>
            <button type="submit" disabled={busy || key.trim() === ''}>
              {t.connect}
            </button>
          </form>
        )}
      </section>

      <section>
        <h2>{t.lastAnswer}</h2>
        {answer === null ? (
          <p>{t.noAnswer}</p>
        ) : (
          <>
            <p>
              {t.status}: {answer.status}
            </p>
            <pre>{JSON.stringify(answer.body, null, 2)}</pre>
          </>
        )}
      </section>
    </main>
  )
}
