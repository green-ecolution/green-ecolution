import { Fragment } from 'react'
import { Link } from '@tanstack/react-router'
import { Alert, AlertContent, AlertDescription, AlertIcon } from '@green-ecolution/ui'
import { imageUrl } from '@/lib/handbook'
import type { Block, Inline } from '@/lib/handbook/types'

const TONE_VARIANT = {
  note: 'info',
  tip: 'success',
  important: 'default',
  warning: 'warning',
} as const

// The block/inline lists are regenerated wholesale from the source document on every
// build and their order is fixed by that document, so the array index is a stable
// identity — hence the react-x/no-array-index-key disables throughout this file.

function Runs({ runs }: { runs: Inline[] }) {
  return (
    <>
      {runs.map((run, i) => {
        switch (run.kind) {
          case 'text':
            // eslint-disable-next-line react-x/no-array-index-key
            return <Fragment key={i}>{run.value}</Fragment>
          case 'code':
            return (
              // eslint-disable-next-line react-x/no-array-index-key
              <code key={i} className="rounded bg-dark-50 px-1 py-0.5 text-[0.9em]">
                {run.value}
              </code>
            )
          case 'strong':
            return (
              // eslint-disable-next-line react-x/no-array-index-key
              <strong key={i} className="font-semibold">
                <Runs runs={run.children} />
              </strong>
            )
          case 'emphasis':
            return (
              // eslint-disable-next-line react-x/no-array-index-key
              <em key={i}>
                <Runs runs={run.children} />
              </em>
            )
          case 'link':
            if (run.target.kind === 'external') {
              return (
                <a
                  // eslint-disable-next-line react-x/no-array-index-key
                  key={i}
                  href={run.target.href}
                  target="_blank"
                  rel="noreferrer"
                  className="text-green-dark underline"
                >
                  <Runs runs={run.children} />
                </a>
              )
            }
            if (run.target.kind === 'app') {
              return (
                // eslint-disable-next-line react-x/no-array-index-key
                <Link key={i} to={run.target.to} className="text-green-dark underline">
                  <Runs runs={run.children} />
                </Link>
              )
            }
            return (
              <Link
                // eslint-disable-next-line react-x/no-array-index-key
                key={i}
                // @ts-expect-error -- /help/$slug is registered by a later task in this plan; the route does not exist yet
                to="/help/$slug"
                // @ts-expect-error -- see above, follows from the same not-yet-registered route
                params={{ slug: run.target.slug }}
                hash={run.target.anchor}
                className="text-green-dark underline"
              >
                <Runs runs={run.children} />
              </Link>
            )
        }
      })}
    </>
  )
}

function One({ block }: { block: Block }) {
  switch (block.kind) {
    case 'heading': {
      const Tag = block.level === 2 ? 'h2' : 'h3'
      const size = block.level === 2 ? 'text-2xl mt-10' : 'text-xl mt-8'
      return (
        <Tag id={block.anchor} className={`font-lato font-bold scroll-mt-24 ${size}`}>
          {block.text}
        </Tag>
      )
    }
    case 'paragraph':
      return (
        <p className="mt-4 leading-relaxed">
          <Runs runs={block.children} />
        </p>
      )
    case 'list':
      return (
        <ul className="mt-4 list-disc space-y-2 pl-6">
          {block.items.map((item, i) => (
            // eslint-disable-next-line react-x/no-array-index-key
            <li key={i}>
              <Runs runs={item} />
            </li>
          ))}
        </ul>
      )
    case 'steps':
      return (
        <ol className="mt-4 list-decimal space-y-2 pl-6 marker:font-semibold marker:text-green-dark">
          {block.items.map((item, i) => (
            // eslint-disable-next-line react-x/no-array-index-key
            <li key={i}>
              <Runs runs={item} />
            </li>
          ))}
        </ol>
      )
    case 'callout':
      return (
        <Alert variant={TONE_VARIANT[block.tone]} className="mt-6 w-full flex gap-3">
          <AlertIcon variant={TONE_VARIANT[block.tone]} />
          <AlertContent>
            <AlertDescription>
              <Blocks blocks={block.children} />
            </AlertDescription>
          </AlertContent>
        </Alert>
      )
    case 'figure':
      return (
        <figure className="mt-8">
          <img
            src={imageUrl(block.image)}
            alt={block.caption}
            className="rounded-2xl border border-dark-100 shadow-cards"
          />
          <figcaption className="mt-2 text-sm text-dark-600">{block.caption}</figcaption>
        </figure>
      )
    case 'table':
      return (
        <div className="mt-6 overflow-x-auto">
          <table className="w-full border-collapse text-left text-sm">
            <thead>
              <tr className="bg-green-light-50">
                {block.head.map((cell, i) => (
                  // eslint-disable-next-line react-x/no-array-index-key
                  <th key={i} className="border-b border-dark-100 px-3 py-2 font-semibold">
                    <Runs runs={cell} />
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, i) => (
                // eslint-disable-next-line react-x/no-array-index-key
                <tr key={i}>
                  {row.map((cell, j) => (
                    // eslint-disable-next-line react-x/no-array-index-key
                    <td key={j} className="border-b border-dark-50 px-3 py-2 align-top">
                      <Runs runs={cell} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )
    case 'code':
      return (
        <pre className="mt-6 overflow-x-auto rounded-xl bg-dark p-4 text-sm text-light">
          <code>{block.value}</code>
        </pre>
      )
  }
}

function Blocks({ blocks }: { blocks: Block[] }) {
  return (
    <>
      {blocks.map((block, i) => (
        // eslint-disable-next-line react-x/no-array-index-key
        <One key={i} block={block} />
      ))}
    </>
  )
}

export default Blocks
