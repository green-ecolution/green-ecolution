import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

const originalStderrWrite = process.stderr.write.bind(process.stderr)
process.stderr.write = (chunk: unknown, ...rest: unknown[]) => {
  if (typeof chunk === 'string' && chunk.includes('Not implemented: navigation')) {
    return true
  }
  // @ts-expect-error -- forwarding the exact overload Node picked
  return originalStderrWrite(chunk, ...rest)
}

afterEach(() => {
  cleanup()
})
