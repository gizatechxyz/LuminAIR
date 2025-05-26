# @gizatech/luminair-react

A React UI component library for LuminAIR proof verification. This library provides a customizable verifier button component that allows users to verify LuminAIR STARK proofs directly in their browser.

## Installation

```bash
npm install @gizatech/luminair-react
```

## Usage

### Next.js Setup

For Next.js projects, you need to configure webpack to handle WASM files and import the CSS:

1. **Import the CSS in your main layout or page:**

```tsx
import '@gizatech/luminair-react/styles.css';
```

2. **Configure Next.js for WASM support:**

**Important**: If you're using Next.js 15+ with Turbopack (`--turbopack` flag), you should disable it for better WASM compatibility. Change your dev script in `package.json`:

```json
{
  "scripts": {
    "dev": "next dev"
  }
}
```

Then create or update your `next.config.js` or `next.config.ts`:

**TypeScript (`next.config.ts`):**
```ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  webpack: (config, { isServer }) => {
    // Handle WASM files
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };

    // Handle .wasm files
    config.module.rules.push({
      test: /\.wasm$/,
      type: "webassembly/async",
    });

    // Fallback for Node.js modules in client-side
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        crypto: false,
      };
    }

    return config;
  },
};

export default nextConfig;
```

**JavaScript (`next.config.js`):**
```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config, { isServer }) => {
    // Handle WASM files
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };

    // Handle .wasm files
    config.module.rules.push({
      test: /\.wasm$/,
      type: 'webassembly/async',
    });

    // Fallback for Node.js modules in client-side
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        crypto: false,
      };
    }

    return config;
  },
};

module.exports = nextConfig;
```

3. **Use the component:**

```tsx
import { VerifyButton } from '@gizatech/luminair-react';

export default function Home() {
  return (
    <VerifyButton
      proofPath="/proof.bin"
      settingsPath="/settings.bin"
    />
  );
}
```

### Basic Usage (Other React frameworks)

```tsx
import { VerifyButton } from '@gizatech/luminair-react';
import '@gizatech/luminair-react/styles.css';

function App() {
  return (
    <VerifyButton
      proofPath="/path/to/proof.bin"
      settingsPath="/path/to/settings.bin"
    />
  );
}
```

### Customized Usage

```tsx
import { VerifyButton } from '@gizatech/luminair-react';
import '@gizatech/luminair-react/styles.css';

function App() {
  return (
    <VerifyButton
      proofPath="/proof.bin"
      settingsPath="/settings.bin"
      title="Custom Verification Title"
      buttonText="VERIFY PROOF"
      author="Your Organization"
      modelDescription="Custom AI Model"
      authorUrl="https://yourwebsite.com"
      className="custom-button-styles"
    />
  );
}
```

## Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `proofPath` | `string` | ✅ | - | Path to the proof file |
| `settingsPath` | `string` | ✅ | - | Path to the settings file |
| `title` | `string` | ❌ | `"Can't be evil."` | Title displayed in the modal |
| `buttonText` | `string` | ❌ | `"VERIFY"` | Text displayed on the button |
| `author` | `string` | ❌ | `"Giza"` | Author name displayed in the modal |
| `modelDescription` | `string` | ❌ | `"Demo model"` | Model description displayed in the modal |
| `authorUrl` | `string` | ❌ | `"https://www.gizatech.xyz/"` | Author URL for the link |
| `className` | `string` | ❌ | - | Custom CSS classes for the button |

## Features

- 🔒 **Secure**: Verification happens entirely in the browser
- 🎨 **Customizable**: All text and styling can be customized
- 📱 **Responsive**: Works on desktop and mobile devices
- 🌙 **Dark Mode**: Built-in dark mode support
- ⚡ **Fast**: Optimized for performance
- 📦 **Lightweight**: Minimal bundle size

## Styling

The component uses Tailwind CSS for styling. Make sure your project has Tailwind CSS configured, or the styles will not work properly.

### Required CSS Variables

The component relies on CSS custom properties for theming. These are included in the imported CSS file, but you can override them:

```css
:root {
  --background: 0 0% 100%;
  --foreground: 0 0% 3.9%;
  --border: 0 0% 89.8%;
  /* ... other variables */
}

.dark {
  --background: 0 0% 3.9%;
  --foreground: 0 0% 98%;
  --border: 0 0% 14.9%;
  /* ... other variables */
}
```

## Dependencies

This library requires the following peer dependencies:

- `react` >= 16.8.0
- `react-dom` >= 16.8.0

## Browser Support

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+

## Troubleshooting

### Next.js Issues

If you encounter WASM-related errors in Next.js:

1. **Disable Turbopack**: Remove `--turbopack` from your dev script in `package.json`
2. Make sure you've configured `next.config.js` as shown above
3. Ensure your proof files are in the `public/` directory
4. Clear cache and restart: `rm -rf .next && npm run dev`

### CSS Not Loading

If styles are not applied:

1. Make sure you're importing the CSS: `import '@gizatech/luminair-react/styles.css'`
2. Verify Tailwind CSS is configured in your project
3. Check that CSS custom properties are defined

### Common Error Messages

- `Module not found: Can't resolve 'luminair_web_bg.wasm'` → Configure Next.js webpack and disable Turbopack
- `Module not found: Can't resolve '@gizatech/luminair-react/styles.css'` → Update to latest version and use correct import path

## License

MIT

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting a PR.

## Support

For support, please open an issue on [GitHub](https://github.com/gizatechxyz/luminair-react/issues) or contact us at [support@gizatech.xyz](mailto:support@gizatech.xyz). 