import type { ReactNode } from 'react'
import { useIsPasswordRevealed } from 'keycloakify/tools/useIsPasswordRevealed'
import type { KcClsx } from 'keycloakify/login/lib/kcClsx'
import type { I18n } from '../i18n'

interface Props {
  kcClsx: KcClsx
  i18n: I18n
  passwordInputId: string
  children: ReactNode
}

export default function PasswordInput(props: Props) {
  const { kcClsx, i18n, passwordInputId, children } = props
  const { msgStr } = i18n
  const { isPasswordRevealed, toggleIsPasswordRevealed } = useIsPasswordRevealed({
    passwordInputId,
  })

  return (
    <div className={kcClsx('kcInputGroup')}>
      {children}
      <button
        type="button"
        className={kcClsx('kcFormPasswordVisibilityButtonClass')}
        aria-label={msgStr(isPasswordRevealed ? 'hidePassword' : 'showPassword')}
        aria-controls={passwordInputId}
        onClick={toggleIsPasswordRevealed}
      >
        <EyeIcon crossed={isPasswordRevealed} />
      </button>
    </div>
  )
}

function EyeIcon(props: { crossed: boolean }) {
  return (
    <svg
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" />
      <circle cx="12" cy="12" r="3" />
      {props.crossed && <path d="m3 3 18 18" />}
    </svg>
  )
}
