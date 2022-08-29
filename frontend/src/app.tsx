import * as React from 'react';
import { createRoot } from 'react-dom/client';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import App from './components/app.tsx';

const container = document.getElementById('app');
const root = createRoot(container);
root.render(<App />);
