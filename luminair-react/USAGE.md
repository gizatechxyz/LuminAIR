# Usage Guide for @gizatech/luminair-react

## Installation

```bash
npm install @gizatech/luminair-react
```

## Prerequisites

### 1. CSS Import

Import the component styles in your main CSS file or component:

```tsx
import '@gizatech/luminair-react/styles.css';
```

### 2. Next.js Configuration (if using Next.js)

Create or update your `next.config.js` file:

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

### 3. Tailwind CSS Setup (Optional but Recommended)

This component library works best with Tailwind CSS. If you don't have it set up:

```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

Update your `tailwind.config.js`:

```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,ts,jsx,tsx}",
    "./node_modules/@gizatech/luminair-react/dist/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

### 4. CSS Variables (Included in styles.css)

The required CSS variables are included in the imported CSS file. You can override them if needed:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 0 0% 3.9%;
    --card: 0 0% 100%;
    --card-foreground: 0 0% 3.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 0 0% 3.9%;
    --primary: 0 0% 9%;
    --primary-foreground: 0 0% 98%;
    --secondary: 0 0% 96.1%;
    --secondary-foreground: 0 0% 9%;
    --muted: 0 0% 96.1%;
    --muted-foreground: 0 0% 45.1%;
    --accent: 0 0% 96.1%;
    --accent-foreground: 0 0% 9%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 0 0% 98%;
    --border: 0 0% 89.8%;
    --input: 0 0% 89.8%;
    --ring: 0 0% 3.9%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 0 0% 3.9%;
    --foreground: 0 0% 98%;
    --card: 0 0% 3.9%;
    --card-foreground: 0 0% 98%;
    --popover: 0 0% 3.9%;
    --popover-foreground: 0 0% 98%;
    --primary: 0 0% 98%;
    --primary-foreground: 0 0% 9%;
    --secondary: 0 0% 14.9%;
    --secondary-foreground: 0 0% 98%;
    --muted: 0 0% 14.9%;
    --muted-foreground: 0 0% 63.9%;
    --accent: 0 0% 14.9%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 0 0% 98%;
    --border: 0 0% 14.9%;
    --input: 0 0% 14.9%;
    --ring: 0 0% 83.1%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
```

## Basic Usage

```tsx
import { VerifyButton } from '@gizatech/luminair-react';
import '@gizatech/luminair-react/styles.css';

function App() {
  return (
    <VerifyButton
      proofPath="/path/to/your/proof.bin"
      settingsPath="/path/to/your/settings.bin"
    />
  );
}
```

## Advanced Usage

### Custom Styling

```tsx
import { VerifyButton } from '@gizatech/luminair-react';
import '@gizatech/luminair-react/styles.css';

function App() {
  return (
    <VerifyButton
      proofPath="/proof.bin"
      settingsPath="/settings.bin"
      title="Custom Verification Portal"
      buttonText="VERIFY PROOF"
      author="Your Organization"
      modelDescription="Custom AI Model v2.0"
      authorUrl="https://yourcompany.com"
      className="bg-blue-600 hover:bg-blue-700 text-white"
    />
  );
}
```

### Multiple Instances

```tsx
import { VerifyButton } from '@gizatech/luminair-react';
import '@gizatech/luminair-react/styles.css';

function ModelGallery() {
  const models = [
    {
      name: "GPT-4 Compatible",
      proofPath: "/models/gpt4/proof.bin",
      settingsPath: "/models/gpt4/settings.bin",
      description: "Large language model for text generation"
    },
    {
      name: "Image Classifier",
      proofPath: "/models/classifier/proof.bin",
      settingsPath: "/models/classifier/settings.bin",
      description: "Convolutional neural network for image classification"
    }
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {models.map((model, index) => (
        <div key={index} className="p-6 border rounded-lg">
          <h3 className="text-lg font-semibold mb-2">{model.name}</h3>
          <p className="text-gray-600 mb-4">{model.description}</p>
          <VerifyButton
            proofPath={model.proofPath}
            settingsPath={model.settingsPath}
            title={`Verify ${model.name}`}
            modelDescription={model.description}
          />
        </div>
      ))}
    </div>
  );
}
```

## Dark Mode Support

The component automatically supports dark mode when you add the `dark` class to your HTML element:

```tsx
// Toggle dark mode
function ThemeToggle() {
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDark]);

  return (
    <button onClick={() => setIsDark(!isDark)}>
      Toggle {isDark ? 'Light' : 'Dark'} Mode
    </button>
  );
}
```

## File Requirements

### Proof Files

Your proof and settings files must be accessible via HTTP. Common setups:

#### Next.js
Place files in the `public/` directory:
```
public/
  proof.bin
  settings.bin
```

#### React (Create React App)
Place files in the `public/` directory:
```
public/
  proof.bin
  settings.bin
```

#### Vite
Place files in the `public/` directory:
```
public/
  proof.bin
  settings.bin
```

### Dynamic Paths

```tsx
function DynamicVerification({ modelId }: { modelId: string }) {
  return (
    <VerifyButton
      proofPath={`/models/${modelId}/proof.bin`}
      settingsPath={`/models/${modelId}/settings.bin`}
      title={`Verify Model ${modelId}`}
    />
  );
}
```

## Error Handling

The component handles errors gracefully and displays them in the verification modal. Common issues:

1. **File not found**: Ensure your proof and settings files are accessible
2. **CORS issues**: Make sure your server allows cross-origin requests for the files
3. **Invalid files**: Verify that your files are valid LuminAIR proof files

## TypeScript Support

The library is fully typed. Import types as needed:

```tsx
import { VerifyButton, VerifyButtonProps } from '@gizatech/luminair-react';

const MyComponent: React.FC<{ config: VerifyButtonProps }> = ({ config }) => {
  return <VerifyButton {...config} />;
};
```

## Performance Considerations

- The verification runs entirely in the browser using WebAssembly
- Large proof files may take time to download and process
- Consider showing loading states while files are being fetched
- The component automatically handles the WASM initialization

## Browser Compatibility

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+
- Requires WebAssembly support

## Troubleshooting

### Component not styled correctly
- Ensure you're importing the CSS: `import '@gizatech/luminair-react/styles.css'`
- Check that Tailwind CSS is properly configured (optional but recommended)
- Verify CSS custom properties are defined

### Verification fails
- Check browser console for errors
- Verify file paths are correct
- Ensure files are valid LuminAIR proof files
- Check network tab for failed requests

### Next.js WASM errors
- Make sure you've configured `next.config.js` as shown above
- Ensure your proof files are in the `public/` directory
- Try restarting your development server after configuration changes

### TypeScript errors
- Ensure you have the latest version of the library
- Check that your TypeScript version is compatible
- Verify all required props are provided 