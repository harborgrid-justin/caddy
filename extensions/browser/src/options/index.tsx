/**
 * CADDY v0.3.0 - Options Page Entry Point
 */

import React from 'react';
import { createRoot } from 'react-dom/client';
import { Options } from './Options';
import './options.css';

const container = document.getElementById('root');
if (container) {
  const root = createRoot(container);
  root.render(
    <React.StrictMode>
      <Options />
    </React.StrictMode>
  );
}
