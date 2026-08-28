import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { KcPage } from './kc.gen'

if (import.meta.env.DEV) {
  const { getKcContextMock } = await import('./login/KcPageStory')
  const pageId = new URLSearchParams(window.location.search).get('page') ?? 'login.ftl'

  window.kcContext = getKcContextMock({
    pageId: pageId as Parameters<typeof getKcContextMock>[0]['pageId'],
    overrides: {},
  })
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {!window.kcContext ? <h1>No Keycloak Context</h1> : <KcPage kcContext={window.kcContext} />}
  </StrictMode>,
)
