import { Clock } from 'lucide-react'

interface SettingsComingSoonProps {
  title: string
  description: string
}

const SettingsComingSoon = ({ title, description }: SettingsComingSoonProps) => (
  <section className="rounded-xl border border-dashed border-dark-200 bg-white p-8 text-center">
    <Clock className="mx-auto size-8 text-dark-400" aria-hidden />
    <h2 className="mt-4 font-lato text-xl font-bold">{title}</h2>
    <p className="mx-auto mt-2 max-w-prose text-sm text-dark-600">{description}</p>
    <p className="mt-4 text-xs font-semibold uppercase tracking-wider text-dark-500">In Arbeit</p>
  </section>
)

export default SettingsComingSoon
