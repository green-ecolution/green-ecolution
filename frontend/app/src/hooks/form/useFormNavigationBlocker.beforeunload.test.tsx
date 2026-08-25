import { describe, it, expect, afterEach } from 'vitest'
import { render, cleanup, waitFor } from '@testing-library/react'
import {
  createBrowserHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router'
import { useFormNavigationBlocker } from './useFormNavigationBlocker'

// The in-app dialog is driven by shouldBlockFn, but the browser's own
// "discard changes?" prompt is driven by enableBeforeUnload, which
// @tanstack/history evaluates without ever consulting shouldBlockFn.
// Only createBrowserHistory installs that listener, so a memory history
// cannot cover this.
function armsNativePrompt() {
  const event = new Event('beforeunload', { cancelable: true })
  window.dispatchEvent(event)
  return event.defaultPrevented
}

function renderFormRoute(isDirty: boolean) {
  function Page() {
    useFormNavigationBlocker({ isDirty, message: 'Änderungen verwerfen?' })
    return <div data-testid="page">Formular</div>
  }

  const router = createRouter({
    routeTree: createRootRoute({ component: Page }),
    history: createBrowserHistory(),
  })

  return render(<RouterProvider router={router} />)
}

describe('useFormNavigationBlocker native beforeunload prompt', () => {
  afterEach(() => {
    cleanup()
  })

  it('does not warn when leaving an untouched form', async () => {
    const { findByTestId } = renderFormRoute(false)
    await findByTestId('page')

    expect(armsNativePrompt()).toBe(false)
  })

  it('warns when leaving a form with unsaved input', async () => {
    const { findByTestId } = renderFormRoute(true)
    await findByTestId('page')

    await waitFor(() => expect(armsNativePrompt()).toBe(true))
  })
})
