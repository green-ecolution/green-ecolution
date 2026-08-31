import type { AuthSession } from './session'

/**
 * Starts an OIDC redirect and resolves only when the visitor is still here --
 * the handover was aborted (custom tab closed, back gesture) or the redirect
 * itself failed. A real navigation tears the document down, so nothing resolves
 * and the caller never continues.
 *
 * oidc-client-ts settles its navigate promise on `pageshow`, which covers a
 * bfcache restore. A standalone PWA never navigates at all -- the authority
 * opens in a custom tab -- so page visibility is the only signal left there.
 */
function startHandover(redirect: () => Promise<void>): Promise<void> {
  return new Promise<void>((resolve) => {
    const done = () => {
      window.removeEventListener('pageshow', done)
      document.removeEventListener('visibilitychange', onVisibilityChange)
      resolve()
    }
    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible') done()
    }

    window.addEventListener('pageshow', done)
    document.addEventListener('visibilitychange', onVisibilityChange)

    // A failed redirect (missing OIDC config) must not strand the caller either.
    redirect().then(done, done)
  })
}

export const startSigninHandover = (
  session: Pick<AuthSession, 'signinRedirect'>,
  returnTo: string,
): Promise<void> => startHandover(() => session.signinRedirect({ returnTo }))

export const startSignoutHandover = (
  session: Pick<AuthSession, 'signoutRedirect'>,
): Promise<void> => startHandover(() => session.signoutRedirect())
