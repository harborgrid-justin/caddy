/**
 * CADDY v0.3.0 - Popup Entry Point
 */

import React from 'react';
import { createRoot } from 'react-dom/client';
import { Popup } from './Popup';
import './popup.css';

const container = document.getElementById('root');
if (container) {
  const root = createRoot(container);
  root.render(
    <React.StrictMode>
      <Popup />
    </React.StrictMode>
  );
}
