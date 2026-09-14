import { describe, expect, it, vi } from 'vitest'
import { createPluginHost, connectToHost, notifyResize } from './handshake'

const ORIGIN = 'https://plugin.example.com'

function fakeIframe() {
  const contentWindow = { postMessage: vi.fn() }
  return { contentWindow } as unknown as HTMLIFrameElement
}

function hello(origin: string, source: unknown) {
  return new MessageEvent('message', {
    data: { ns: 'green-ecolution', v: 1, type: 'ge:hello', payload: {} },
    origin,
    source: source as Window,
  })
}

describe('createPluginHost', () => {
  const context = {
    locale: 'de' as const,
    theme: 'light' as const,
    user: { displayName: 'Test' },
    plugin: { slug: 'acme' },
  }

  it('answers a hello from the expected origin with the context', () => {
    const iframe = fakeIframe()
    createPluginHost(iframe, { origin: ORIGIN, context })

    window.dispatchEvent(hello(ORIGIN, iframe.contentWindow))

    expect(iframe.contentWindow!.postMessage).toHaveBeenCalledWith(
      { ns: 'green-ecolution', v: 1, type: 'ge:init', payload: context },
      ORIGIN,
    )
  })

  it('ignores a hello from a foreign origin', () => {
    const iframe = fakeIframe()
    createPluginHost(iframe, { origin: ORIGIN, context })

    window.dispatchEvent(hello('https://evil.example.com', iframe.contentWindow))

    expect(iframe.contentWindow!.postMessage).not.toHaveBeenCalled()
  })

  it('ignores a message from another window on the right origin', () => {
    const iframe = fakeIframe()
    createPluginHost(iframe, { origin: ORIGIN, context })

    window.dispatchEvent(hello(ORIGIN, { postMessage: vi.fn() }))

    expect(iframe.contentWindow!.postMessage).not.toHaveBeenCalled()
  })

  it('ignores envelopes of an unknown namespace or version', () => {
    const iframe = fakeIframe()
    createPluginHost(iframe, { origin: ORIGIN, context })

    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'something-else', v: 1, type: 'ge:hello', payload: {} },
        origin: ORIGIN,
        source: iframe.contentWindow as Window,
      }),
    )
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 99, type: 'ge:hello', payload: {} },
        origin: ORIGIN,
        source: iframe.contentWindow as Window,
      }),
    )

    expect(iframe.contentWindow!.postMessage).not.toHaveBeenCalled()
  })

  it('stops listening after teardown', () => {
    const iframe = fakeIframe()
    const dispose = createPluginHost(iframe, { origin: ORIGIN, context })
    dispose()

    window.dispatchEvent(hello(ORIGIN, iframe.contentWindow))

    expect(iframe.contentWindow!.postMessage).not.toHaveBeenCalled()
  })

  it('reports a resize to the optional callback', () => {
    const iframe = fakeIframe()
    const onResize = vi.fn()
    createPluginHost(iframe, { origin: ORIGIN, context, onResize })

    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:resize', payload: { height: 420 } },
        origin: ORIGIN,
        source: iframe.contentWindow as Window,
      }),
    )

    expect(onResize).toHaveBeenCalledWith(420)
  })
})

describe('connectToHost', () => {
  it('sends a hello to the parent and resolves with the matching init payload', async () => {
    const postMessage = vi.spyOn(window.parent, 'postMessage')

    const pending = connectToHost()

    expect(postMessage).toHaveBeenCalledWith(
      { ns: 'green-ecolution', v: 1, type: 'ge:hello', payload: {} },
      '*',
    )

    const context = {
      locale: 'en' as const,
      theme: 'dark' as const,
      user: { displayName: 'Test' },
      plugin: { slug: 'acme' },
    }
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:init', payload: context },
        source: window.parent,
      }),
    )

    await expect(pending).resolves.toEqual(context)
    postMessage.mockRestore()
  })

  it('ignores unrelated envelopes before resolving', async () => {
    const pending = connectToHost()

    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:resize', payload: { height: 1 } },
        source: window.parent,
      }),
    )
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:init', payload: { plugin: { slug: 'x' } } },
        source: window.parent,
      }),
    )

    await expect(pending).resolves.toEqual({ plugin: { slug: 'x' } })
  })

  it('ignores a ge:init from a window other than the parent', async () => {
    const pending = connectToHost()
    const context = {
      locale: 'en' as const,
      theme: 'dark' as const,
      user: { displayName: 'Test' },
      plugin: { slug: 'acme' },
    }

    window.dispatchEvent(
      new MessageEvent('message', {
        data: {
          ns: 'green-ecolution',
          v: 1,
          type: 'ge:init',
          payload: { plugin: { slug: 'not-it' } },
        },
        source: { postMessage: vi.fn() } as unknown as Window,
      }),
    )
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:init', payload: context },
        source: window.parent,
      }),
    )

    await expect(pending).resolves.toEqual(context)
  })

  it('repeats the hello until the host answers', async () => {
    vi.useFakeTimers()
    const postMessage = vi.spyOn(window.parent, 'postMessage')

    const pending = connectToHost({ retryIntervalMs: 100 })
    expect(postMessage).toHaveBeenCalledTimes(1)

    // The host attaches its listener late, so the first hello is lost.
    await vi.advanceTimersByTimeAsync(250)
    expect(postMessage.mock.calls.length).toBeGreaterThan(1)

    window.dispatchEvent(
      new MessageEvent('message', {
        data: { ns: 'green-ecolution', v: 1, type: 'ge:init', payload: { plugin: { slug: 'x' } } },
        source: window.parent,
      }),
    )
    await expect(pending).resolves.toEqual({ plugin: { slug: 'x' } })

    const afterResolve = postMessage.mock.calls.length
    await vi.advanceTimersByTimeAsync(500)
    expect(postMessage.mock.calls.length).toBe(afterResolve)

    postMessage.mockRestore()
    vi.useRealTimers()
  })

  it('rejects when the host never answers', async () => {
    vi.useFakeTimers()

    const pending = connectToHost({ timeoutMs: 1000 })
    const assertion = expect(pending).rejects.toThrow(/did not answer the handshake/)
    await vi.advanceTimersByTimeAsync(1000)
    await assertion

    vi.useRealTimers()
  })
})

describe('notifyResize', () => {
  it('posts a resize envelope to the parent window', () => {
    const postMessage = vi.spyOn(window.parent, 'postMessage')

    notifyResize(123)

    expect(postMessage).toHaveBeenCalledWith(
      { ns: 'green-ecolution', v: 1, type: 'ge:resize', payload: { height: 123 } },
      '*',
    )
    postMessage.mockRestore()
  })
})
