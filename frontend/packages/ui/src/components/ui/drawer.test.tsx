import { act, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Drawer, DrawerContent, DrawerTitle } from './drawer'

const flushFrame = () =>
  act(async () => {
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
  })

describe('Drawer', () => {
  it('keeps the page behind a non-modal drawer interactive (no body pointer-events lock)', async () => {
    render(
      <Drawer open modal={false}>
        <DrawerContent showOverlay={false}>
          <DrawerTitle>Details</DrawerTitle>
          <p>content</p>
        </DrawerContent>
      </Drawer>,
    )

    await flushFrame()

    expect(screen.getByText('content')).toBeInTheDocument()
    expect(document.body.style.pointerEvents).not.toBe('none')
  })
})
