import logoUrl from '../assets/logo-with-text-white.svg'

interface Props {
  variant: 'side' | 'header'
  headline: string
  intro: string
}

/**
 * Deepens toward the bottom. The palette is counter-intuitive here:
 * --green-dark-900 is LIGHTER than --green-dark (oklch lightness 0.588 vs
 * 0.524), so listing them in numeric order washes the panel out.
 */
const PANEL_BACKGROUND =
  'bg-[linear-gradient(152deg,var(--green-dark-900)_0%,var(--green-dark)_46%,color-mix(in_oklch,var(--green-dark)_74%,black)_100%)]'

export default function BrandPanel(props: Props) {
  const { variant, headline, intro } = props

  if (variant === 'header') {
    return (
      <div
        className={`relative flex h-36 items-center overflow-hidden px-6 lg:hidden ${PANEL_BACKGROUND}`}
      >
        <Atmosphere />
        <img src={logoUrl} alt="Green Ecolution" className="relative z-10 w-52 max-w-full" />
      </div>
    )
  }

  return (
    <div
      className={`relative hidden overflow-hidden lg:flex lg:w-[51%] lg:shrink-0 lg:flex-col lg:justify-between lg:p-14 ${PANEL_BACKGROUND}`}
    >
      <Atmosphere />

      <img src={logoUrl} alt="Green Ecolution" className="relative z-10 w-72 max-w-full" />

      <div className="relative z-10 max-w-lg">
        <h2 className="font-lato text-[2.4rem] leading-[1.12] font-bold text-balance text-white">
          {headline}
        </h2>
        <p className="mt-5 text-[0.95rem] leading-relaxed text-white/75">{intro}</p>
      </div>
    </div>
  )
}

/**
 * A warm highlight with no visible edge plus a fine dot grid. An earlier
 * attempt put the website's blob path here and it read as a pale wedge cropped
 * by the corner; light without a boundary gives the flat green depth instead.
 */
function Atmosphere() {
  return (
    <>
      <span
        aria-hidden="true"
        className="pointer-events-none absolute top-[-10%] right-[-15%] h-[70%] w-[80%] bg-[radial-gradient(closest-side,rgba(236,245,180,0.28),rgba(236,245,180,0.08)_55%,transparent)] motion-safe:animate-[ge-orb_20s_ease-in-out_infinite_alternate]"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 bg-[radial-gradient(rgba(255,255,255,0.07)_1px,transparent_1px)] bg-[length:22px_22px]"
      />
    </>
  )
}
