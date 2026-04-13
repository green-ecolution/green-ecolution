import type { Meta, StoryObj } from '@storybook/react-vite'
import { useEffect, useRef, useState } from 'react'
import { CameraOff, CircleAlert, ShieldAlert } from 'lucide-react'
import { CameraViewport, type CameraViewportState } from '../src/components/ui/camera-viewport'
import { Loading } from '../src/components/ui/spinner'
import { Button } from '../src/components/ui/button'

const meta: Meta<typeof CameraViewport> = {
  title: 'UI/CameraViewport',
  component: CameraViewport,
  tags: ['autodocs'],
  argTypes: {
    state: {
      control: 'select',
      options: ['inactive', 'loading', 'scanning', 'success', 'error'],
    },
    flash: {
      control: 'boolean',
    },
    ariaLabel: {
      control: 'text',
    },
  },
  parameters: {
    docs: {
      description: {
        component:
          'Visueller Container für Kamera-Streams (z. B. QR-Scanner). Rendert Spotlight, Eck-Brackets und animierte Scan-Linie. Der Video-Stream wird via `videoRef` von außen gesteuert.',
      },
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

const Frame = ({ children }: { children: React.ReactNode }) => (
  <div className="max-w-md">{children}</div>
)

const overlayFor = (state: CameraViewportState): React.ReactNode => {
  switch (state) {
    case 'loading':
      return (
        <span className="flex flex-col items-center gap-3 text-white/80">
          <Loading size="lg" label="" />
          <span className="text-sm">Kamera wird gestartet …</span>
        </span>
      )
    case 'error':
      return <CircleAlert aria-hidden="true" className="size-12 text-white/40" />
    case 'inactive':
      return <CameraOff aria-hidden="true" className="size-12 text-white/40" />
    default:
      return null
  }
}

export const Inactive: Story = {
  args: {
    state: 'inactive',
    overlay: <CameraOff aria-hidden="true" className="size-12 text-white/40" />,
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport {...args} videoRef={videoRef} />
      </Frame>
    )
  },
}

export const Loading_: Story = {
  name: 'Loading',
  args: {
    state: 'loading',
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport {...args} videoRef={videoRef} overlay={overlayFor('loading')} />
      </Frame>
    )
  },
}

export const Scanning: Story = {
  args: {
    state: 'scanning',
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport {...args} videoRef={videoRef} />
      </Frame>
    )
  },
}

export const Success: Story = {
  args: {
    state: 'success',
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport {...args} videoRef={videoRef} />
      </Frame>
    )
  },
}

export const Denied: Story = {
  args: {
    state: 'error',
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport
          {...args}
          videoRef={videoRef}
          overlay={<CameraOff aria-hidden="true" className="size-12 text-white/40" />}
        />
      </Frame>
    )
  },
}

export const Unsupported: Story = {
  args: {
    state: 'error',
  },
  render: (args) => {
    const videoRef = useRef<HTMLVideoElement>(null)
    return (
      <Frame>
        <CameraViewport
          {...args}
          videoRef={videoRef}
          overlay={<ShieldAlert aria-hidden="true" className="size-12 text-white/40" />}
        />
      </Frame>
    )
  },
}

export const FlashOnScan: Story = {
  parameters: {
    docs: {
      description: {
        story:
          'Triggert den one-shot Scan-Flash inkl. Corner-Pulse — bei jedem Klick wird die Animation neu abgespielt.',
      },
    },
  },
  render: () => {
    const videoRef = useRef<HTMLVideoElement>(null)
    const [flash, setFlash] = useState(false)

    useEffect(() => {
      if (!flash) return
      const id = window.setTimeout(() => setFlash(false), 500)
      return () => window.clearTimeout(id)
    }, [flash])

    return (
      <Frame>
        <div className="flex flex-col gap-4">
          <CameraViewport state="scanning" videoRef={videoRef} flash={flash} />
          <Button onClick={() => setFlash(true)}>Scan-Flash auslösen</Button>
        </div>
      </Frame>
    )
  },
}

export const LiveCamera: Story = {
  parameters: {
    docs: {
      description: {
        story:
          'Verbindet eine echte Kamera über getUserMedia. Demonstriert das Verhalten mit live-Stream — funktioniert nur in einem sicheren Kontext (HTTPS oder localhost).',
      },
    },
  },
  render: () => {
    const videoRef = useRef<HTMLVideoElement>(null)
    const [state, setState] = useState<CameraViewportState>('inactive')
    const [error, setError] = useState<string | null>(null)

    const start = async () => {
      setError(null)
      setState('loading')
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: { ideal: 'environment' } },
          audio: false,
        })
        if (videoRef.current) {
          videoRef.current.srcObject = stream
        }
        setState('scanning')
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        setState('error')
      }
    }

    const stop = () => {
      const stream = videoRef.current?.srcObject as MediaStream | null
      stream?.getTracks().forEach((t) => t.stop())
      if (videoRef.current) videoRef.current.srcObject = null
      setState('inactive')
    }

    useEffect(() => stop, [])

    return (
      <Frame>
        <div className="flex flex-col gap-4">
          <CameraViewport
            state={state}
            videoRef={videoRef}
            overlay={overlayFor(state)}
            ariaLabel="Live-Kamera-Vorschau"
          />
          <div className="flex gap-2">
            <Button size="sm" onClick={() => void start()} disabled={state !== 'inactive'}>
              Kamera starten
            </Button>
            <Button size="sm" variant="outline" onClick={stop} disabled={state === 'inactive'}>
              Stoppen
            </Button>
          </div>
          {error && (
            <p className="text-xs font-mono text-red break-all" role="alert">
              {error}
            </p>
          )}
        </div>
      </Frame>
    )
  },
}

export const AllStates: Story = {
  render: () => {
    const stubRef = useRef<HTMLVideoElement>(null)
    const states: { state: CameraViewportState; label: string; overlay?: React.ReactNode }[] = [
      { state: 'inactive', label: 'Inactive', overlay: overlayFor('inactive') },
      { state: 'loading', label: 'Loading', overlay: overlayFor('loading') },
      { state: 'scanning', label: 'Scanning' },
      { state: 'success', label: 'Success' },
      { state: 'error', label: 'Error', overlay: overlayFor('error') },
    ]
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
        {states.map(({ state, label, overlay }) => (
          <div key={state} className="flex flex-col gap-2">
            <h4 className="text-sm font-medium text-muted-foreground">{label}</h4>
            <CameraViewport state={state} videoRef={stubRef} overlay={overlay} />
          </div>
        ))}
      </div>
    )
  },
}
