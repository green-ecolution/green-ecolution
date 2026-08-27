import logoUrl from '../assets/logo-with-text-white.svg'

type Props = {
  variant: 'side' | 'header'
  claim: string
}

export default function BrandPanel(props: Props) {
  const { variant, claim } = props
  const isSide = variant === 'side'

  return (
    <div
      className={
        isSide
          ? 'relative hidden overflow-hidden bg-[linear-gradient(160deg,var(--green-dark),var(--green-dark-900))] lg:flex lg:w-[42%] lg:flex-col lg:justify-center lg:px-14'
          : 'relative flex h-40 items-center overflow-hidden bg-[linear-gradient(160deg,var(--green-dark),var(--green-dark-900))] px-6 lg:hidden'
      }
    >
      <Orbs />
      <Silhouette />
      <div className="relative z-10">
        <img
          src={logoUrl}
          alt="Green Ecolution"
          className={isSide ? 'mb-8 w-64 max-w-full' : 'w-44 max-w-full'}
        />
        {isSide && (
          <p className="max-w-sm text-lg leading-relaxed text-white/80">{claim}</p>
        )}
      </div>
    </div>
  )
}

function Orbs() {
  return (
    <>
      <span className="pointer-events-none absolute -top-[30%] -right-[20%] h-[60vmax] w-[60vmax] rounded-full bg-[radial-gradient(circle,rgba(255,255,255,0.10)_0%,transparent_70%)] motion-safe:animate-[ge-orb_9s_ease-in-out_infinite_alternate]" />
      <span className="pointer-events-none absolute -bottom-[35%] -left-[25%] h-[55vmax] w-[55vmax] rounded-full bg-[radial-gradient(circle,rgba(255,255,255,0.07)_0%,transparent_70%)] motion-safe:animate-[ge-orb_11s_ease-in-out_1s_infinite_alternate-reverse]" />
    </>
  )
}

function Silhouette() {
  return (
    <svg
      className="pointer-events-none absolute -right-16 bottom-0 h-3/4 w-auto text-white/[0.08] max-[380px]:hidden"
      viewBox="0 0 200 240"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M100 8c-30 0-54 21-54 47 0 9 3 17 8 24-13 6-22 19-22 34 0 17 12 31 28 35-3 5-5 11-5 17 0 18 16 33 36 33h6v34h6v-34h6c20 0 36-15 36-33 0-6-2-12-5-17 16-4 28-18 28-35 0-15-9-28-22-34 5-7 8-15 8-24 0-26-24-47-54-47z" />
    </svg>
  )
}
