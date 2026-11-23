import * as React from 'react'
import { cn } from '@/lib/utils'

interface EmptyStateBenefit {
  icon: React.ReactNode
  title: string
  text: string
}

interface EmptyStateProps extends React.HTMLAttributes<HTMLElement> {
  title: string
  description: string
  /** Usually one button; absent where the viewer may not create anything. */
  action?: React.ReactNode
  /** A small picture of the filled state. `label` is read to screen readers in its place. */
  example?: { caption: string; label: string; content: React.ReactNode }
  benefits?: { title: string; items: EmptyStateBenefit[] }
  /** One line of fine print at the bottom. */
  note?: React.ReactNode
}

function EmptyState({
  className,
  title,
  description,
  action,
  example,
  benefits,
  note,
  ...props
}: EmptyStateProps) {
  const titleId = React.useId()

  return (
    <section
      aria-labelledby={titleId}
      className={cn(
        'overflow-hidden rounded-xl border border-dark-50 bg-white shadow-cards',
        className,
      )}
      {...props}
    >
      <div className="flex flex-wrap items-center gap-x-12 gap-y-8 p-6 md:p-10">
        <div className="flex min-w-0 flex-[1_1_20rem] flex-col items-start gap-3">
          <h3 id={titleId} className="font-lato text-xl font-bold leading-tight md:text-[22px]">
            {title}
          </h3>
          <p className="max-w-[52ch] text-dark-800">{description}</p>
          {action && <div className="mt-2">{action}</div>}
        </div>

        {example && (
          <div className="flex min-w-0 max-w-sm flex-[1_1_18rem] flex-col gap-2">
            <p className="mb-1 text-sm font-bold text-dark-600">{example.caption}</p>
            <div role="img" aria-label={example.label}>
              <div aria-hidden="true">{example.content}</div>
            </div>
          </div>
        )}
      </div>

      {benefits && (
        <div className="border-t border-dark-50 px-6 pb-8 pt-7 md:px-10">
          <h4 className="mb-5 font-lato text-base font-bold">{benefits.title}</h4>
          <ul className="flex flex-wrap gap-x-10 gap-y-6">
            {benefits.items.map((item) => (
              <li key={item.title} className="flex min-w-0 flex-[1_1_12rem] gap-3">
                <span aria-hidden="true" className="mt-0.5 shrink-0 text-green-dark">
                  {item.icon}
                </span>
                <div>
                  <p className="font-bold">{item.title}</p>
                  <p className="mt-0.5 text-dark-800">{item.text}</p>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {note && (
        <p className="border-t border-dark-50 bg-dark-50/60 px-6 py-4 text-dark-800 md:px-10">
          {note}
        </p>
      )}
    </section>
  )
}

export { EmptyState }
export type { EmptyStateProps, EmptyStateBenefit }
