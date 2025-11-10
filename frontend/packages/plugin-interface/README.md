# Green Ecolution - Plugin Interface

**Type-safe plugin interface for the Green Ecolution platform.**

This package provides the shared contracts, types, and utilities for developing plugins that extend the Green Ecolution frontend. Plugins are dynamically loaded at runtime using **Module Federation** and can contribute routes, components, and custom functionality.

## Features

- ✅ Type-safe plugin context and contracts
- ✅ React Context API for plugin-host communication
- ✅ Authentication token access for API calls
- ✅ Peer dependency on React 19
- ✅ Published as `@green-ecolution/plugin-interface`

## Installation

Install the package using your preferred package manager:

```bash
npm install @green-ecolution/plugin-interface
# or
yarn add @green-ecolution/plugin-interface
# or
pnpm add @green-ecolution/plugin-interface
```

## Plugin Architecture

Plugins are loaded dynamically at runtime via Module Federation:

```
┌─────────────────────────────────────┐
│      Green Ecolution Host App       │
│                                     │
│  ┌───────────────────────────────┐  │
│  │   Plugin Loader & Registry    │  │
│  └───────────────────────────────┘  │
│              │                      │
│              ├──────┐               │
│              ▼      ▼               │
│         Plugin A  Plugin B          │
│                                     │
└─────────────────────────────────────┘
```

**Plugin Lifecycle:**

1. **Registration** - Plugin registers with backend API (`POST /v1/plugin`)
2. **Discovery** - Host app fetches plugin list from backend
3. **Loading** - Plugin JavaScript bundle is loaded via Module Federation
4. **Initialization** - Plugin receives `PluginContext` with auth token
5. **Activation** - Plugin routes and components become available

## API Reference

### `PluginContext`

The context object provided by the host application to plugins:

```typescript
interface PluginContext {
  authToken: string // JWT token for authenticated API requests
}
```

### `PluginProvider`

React context provider that wraps your plugin and provides access to the `PluginContext`.

**Props:**

```typescript
interface PluginProviderProps extends React.PropsWithChildren {
  authToken: string // JWT token from the host app
}
```

**Usage in Host App:**

```tsx
import { PluginProvider } from '@green-ecolution/plugin-interface'

function HostApp() {
  const authToken = useAuthStore((state) => state.token)

  return (
    <PluginProvider authToken={authToken}>
      <RemotePluginComponent />
    </PluginProvider>
  )
}
```

### `usePluginContext`

React hook to access the `PluginContext` from within a plugin component.

**Returns:** `PluginContext`

**Throws:** Error if used outside of `PluginProvider`

**Usage in Plugin:**

```tsx
import { usePluginContext } from '@green-ecolution/plugin-interface'

export function MyPluginComponent() {
  const { authToken } = usePluginContext()

  // Use authToken for API requests
  const fetchData = async () => {
    const response = await fetch('/api/v1/data', {
      headers: {
        Authorization: `Bearer ${authToken}`,
      },
    })
    return response.json()
  }

  return <div>My Plugin Content</div>
}
```

## Creating a Plugin

### 1. Project Setup

Initialize a new Vite + React + TypeScript project:

```bash
pnpm create vite my-plugin --template react-ts
cd my-plugin
pnpm install
pnpm add @green-ecolution/plugin-interface
```

### 2. Configure Module Federation

Add Module Federation to your `vite.config.ts`:

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import federation from '@originjs/vite-plugin-federation'

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'my_plugin',
      filename: 'remoteEntry.js',
      exposes: {
        './Plugin': './src/Plugin.tsx',
      },
      shared: ['react', 'react-dom', '@green-ecolution/plugin-interface'],
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
})
```

### 3. Create Plugin Component

Create `src/Plugin.tsx`:

```tsx
import { usePluginContext } from '@green-ecolution/plugin-interface'
import { useEffect, useState } from 'react'

export default function MyPlugin() {
  const { authToken } = usePluginContext()
  const [data, setData] = useState(null)

  useEffect(() => {
    // Make authenticated API requests
    fetch('https://app.green-ecolution.de/api/v1/tree', {
      headers: {
        Authorization: `Bearer ${authToken}`,
      },
    })
      .then((res) => res.json())
      .then(setData)
  }, [authToken])

  return (
    <div>
      <h1>My Custom Plugin</h1>
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  )
}
```

### 4. Build and Deploy

```bash
pnpm run build
```

Deploy the `dist/` folder to a web server and note the URL.

### 5. Register with Backend

Register your plugin with the Green Ecolution backend:

```bash
curl -X POST https://app.green-ecolution.de/api/v1/plugin \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "My Plugin",
    "slug": "my-plugin",
    "url": "https://your-server.com/remoteEntry.js",
    "version": "1.0.0"
  }'
```

The host app will now discover and load your plugin automatically!

## Plugin Development Guidelines

### Best Practices

1. **Always use `usePluginContext`** to access the auth token
2. **Handle errors gracefully** - plugins should not crash the host app
3. **Keep bundle size small** - use code splitting and lazy loading
4. **Follow React best practices** - use hooks, avoid side effects in render
5. **Test in isolation** - develop and test your plugin independently

### Security Considerations

- **Never expose secrets** in plugin code or configuration
- **Validate all user input** before sending to the backend
- **Use the provided auth token** - don't manage authentication yourself
- **Respect user permissions** - plugins inherit the user's role and access level

### Styling

Plugins can use:

- **Tailwind CSS classes** (if the host app includes Tailwind)
- **CSS Modules** for scoped styles
- **Inline styles** for simple cases

Avoid global CSS that might conflict with the host app.

## Troubleshooting

### Plugin doesn't load

- Check that the `url` in plugin registration points to a valid `remoteEntry.js`
- Verify CORS headers allow loading from the plugin host
- Check browser console for Module Federation errors

### Auth token is undefined

- Ensure your component is wrapped in `<PluginProvider>`
- Verify the host app is passing a valid `authToken` prop

### Type errors with React

- Ensure you're using React 19 (peer dependency)
- Check that `@green-ecolution/plugin-interface` is in `peerDependencies` not `dependencies`

## Examples

### Example: Tree Map Plugin

A plugin that displays trees on a custom map:

```tsx
import { usePluginContext } from '@green-ecolution/plugin-interface'
import { useQuery } from '@tanstack/react-query'
import { MapContainer, TileLayer, Marker } from 'react-leaflet'

export default function TreeMapPlugin() {
  const { authToken } = usePluginContext()

  const { data: trees } = useQuery({
    queryKey: ['trees'],
    queryFn: async () => {
      const res = await fetch('/api/v1/tree', {
        headers: { Authorization: `Bearer ${authToken}` },
      })
      return res.json()
    },
  })

  return (
    <MapContainer center={[54.78, 9.44]} zoom={13}>
      <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" />
      {trees?.data.map((tree) => (
        <Marker key={tree.id} position={[tree.latitude, tree.longitude]} />
      ))}
    </MapContainer>
  )
}
```

## Contributing

Contributions are welcome! To propose changes to the plugin interface:

1. Fork the [green-ecolution/frontend](https://github.com/green-ecolution/frontend) repository
2. Create a feature branch from `develop`
3. Make your changes in `packages/plugin-interface/`
4. Follow [Conventional Commits](https://www.conventionalcommits.org/)
5. Open a Pull Request to `develop`

## Links

- 🌐 [Green Ecolution Website](https://green-ecolution.de)
- 📘 [API Documentation](https://app.green-ecolution.de/api/v1/swagger/index.html)
- 🧑‍💻 [GitHub Repository](https://github.com/green-ecolution/frontend)
- 🖥️ [Live Demo](https://demo.green-ecolution.de)

## License

AGPL-3.0-only

---

**Maintained by the Green Ecolution Team**

For questions or support, please open an issue on GitHub.
