import { BadgeCheck } from 'lucide-react'

const KeyFacts = () => {
  const numbers = [
    'Monitoring des Bewässerungszustands',
    'Gruppierung von Bäumen anhand von Standortbedingungen',
    'Interpretation der Messwerte der Sensordaten in ein Ampelsystem',
    'Vereinfachte & Digitalisierte Einsatzplanung',
    'Dynamische Routenplanung von Bewässerungsfahrten',
    'Langzeitauswertung von Daten',
  ]
  return (
    <section className="container mt-20 lg:mt-28">
      <h2 className="font-semibold text-dark-800 mb-6">Wobei genau hilft Ihnen Green Ecolution?</h2>
      <ul className="grid grid-cols-1 gap-y-4 lg:grid-cols-2 lg:gap-x-10">
        {numbers.map((number) => (
          <li key={number} className="flex items-center gap-x-4">
            <BadgeCheck className="text-green-light w-8 h-8 shrink-0" />
            <p className="font-semibold text-lg">{number}</p>
          </li>
        ))}
      </ul>
    </section>
  )
}

export default KeyFacts
