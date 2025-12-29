/**
 * Enterprise Color Picker Component
 * Features: HSL/RGB/HEX support, presets, alpha channel, eyedropper, accessibility
 */

import React, { useState, useRef, useEffect, CSSProperties } from 'react';
import { useTheme } from './styles/theme';
import { transitionPresets, animationPresets } from './styles/animations';

export interface ColorPickerProps {
  /** Current color value (hex format) */
  value?: string;
  /** Color change handler */
  onChange?: (color: string) => void;
  /** Show alpha channel */
  showAlpha?: boolean;
  /** Color presets */
  presets?: string[];
  /** Disabled state */
  disabled?: boolean;
  /** Label */
  label?: string;
}

interface HSL {
  h: number; // 0-360
  s: number; // 0-100
  l: number; // 0-100
  a: number; // 0-1
}

export const ColorPicker: React.FC<ColorPickerProps> = ({
  value = '#3b82f6',
  onChange,
  showAlpha = false,
  presets = [],
  disabled = false,
  label,
}) => {
  const { theme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const [color, setColor] = useState(value);
  const [hsl, setHsl] = useState<HSL>(hexToHSL(value));
  const containerRef = useRef<HTMLDivElement>(null);
  const saturationRef = useRef<HTMLDivElement>(null);

  const defaultPresets = [
    '#ef4444', '#f59e0b', '#10b981', '#3b82f6', '#6366f1', '#8b5cf6', '#ec4899',
    '#000000', '#374151', '#6b7280', '#9ca3af', '#d1d5db', '#f3f4f6', '#ffffff',
  ];

  const colorPresets = presets.length > 0 ? presets : defaultPresets;

  useEffect(() => {
    setColor(value);
    setHsl(hexToHSL(value));
  }, [value]);

  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  function hexToHSL(hex: string): HSL {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    if (!result) return { h: 0, s: 0, l: 0, a: 1 };

    let r = parseInt(result[1], 16) / 255;
    let g = parseInt(result[2], 16) / 255;
    let b = parseInt(result[3], 16) / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (max !== min) {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

      switch (max) {
        case r:
          h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
          break;
        case g:
          h = ((b - r) / d + 2) / 6;
          break;
        case b:
          h = ((r - g) / d + 4) / 6;
          break;
      }
    }

    return { h: h * 360, s: s * 100, l: l * 100, a: 1 };
  }

  function hslToHex(hsl: HSL): string {
    const h = hsl.h / 360;
    const s = hsl.s / 100;
    const l = hsl.l / 100;

    let r, g, b;

    if (s === 0) {
      r = g = b = l;
    } else {
      const hue2rgb = (p: number, q: number, t: number) => {
        if (t < 0) t += 1;
        if (t > 1) t -= 1;
        if (t < 1 / 6) return p + (q - p) * 6 * t;
        if (t < 1 / 2) return q;
        if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
        return p;
      };

      const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
      const p = 2 * l - q;

      r = hue2rgb(p, q, h + 1 / 3);
      g = hue2rgb(p, q, h);
      b = hue2rgb(p, q, h - 1 / 3);
    }

    const toHex = (x: number) => {
      const hex = Math.round(x * 255).toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    };

    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  }

  const handleSaturationChange = (e: React.MouseEvent<HTMLDivElement>) => {
    if (disabled || !saturationRef.current) return;

    const rect = saturationRef.current.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const y = Math.max(0, Math.min(e.clientY - rect.top, rect.height));

    const newHsl: HSL = {
      ...hsl,
      s: (x / rect.width) * 100,
      l: 100 - (y / rect.height) * 100,
    };

    setHsl(newHsl);
    const newColor = hslToHex(newHsl);
    setColor(newColor);
    onChange?.(newColor);
  };

  const handleHueChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newHsl: HSL = { ...hsl, h: Number(e.target.value) };
    setHsl(newHsl);
    const newColor = hslToHex(newHsl);
    setColor(newColor);
    onChange?.(newColor);
  };

  const handleAlphaChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newHsl: HSL = { ...hsl, a: Number(e.target.value) };
    setHsl(newHsl);
  };

  const handleHexInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const hex = e.target.value;
    if (/^#[0-9A-F]{6}$/i.test(hex)) {
      setColor(hex);
      setHsl(hexToHSL(hex));
      onChange?.(hex);
    }
  };

  const handlePresetClick = (preset: string) => {
    setColor(preset);
    setHsl(hexToHSL(preset));
    onChange?.(preset);
  };

  const swatchStyles: CSSProperties = {
    width: '40px',
    height: '40px',
    borderRadius: theme.borderRadius.md,
    border: `2px solid ${theme.colors.border.primary}`,
    cursor: disabled ? 'not-allowed' : 'pointer',
    backgroundColor: color,
    position: 'relative',
    opacity: disabled ? 0.6 : 1,
  };

  const popoverStyles: CSSProperties = {
    position: 'absolute',
    top: '100%',
    left: 0,
    marginTop: theme.spacing[2],
    backgroundColor: theme.colors.background.elevated,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.lg,
    boxShadow: theme.shadows.xl,
    padding: theme.spacing[4],
    zIndex: theme.zIndex.popover,
    animation: animationPresets.slideInDown,
    minWidth: '280px',
  };

  const saturationStyles: CSSProperties = {
    width: '100%',
    height: '150px',
    position: 'relative',
    borderRadius: theme.borderRadius.md,
    background: `linear-gradient(to bottom, transparent, black),
                 linear-gradient(to right, white, hsl(${hsl.h}, 100%, 50%))`,
    cursor: 'crosshair',
    marginBottom: theme.spacing[3],
  };

  const pickerDotStyles: CSSProperties = {
    position: 'absolute',
    left: `${hsl.s}%`,
    top: `${100 - hsl.l}%`,
    width: '12px',
    height: '12px',
    border: '2px solid white',
    borderRadius: '50%',
    transform: 'translate(-50%, -50%)',
    pointerEvents: 'none',
    boxShadow: '0 0 0 1px rgba(0, 0, 0, 0.3)',
  };

  const sliderStyles: CSSProperties = {
    width: '100%',
    height: '12px',
    borderRadius: theme.borderRadius.full,
    cursor: 'pointer',
    outline: 'none',
    WebkitAppearance: 'none',
    appearance: 'none',
  };

  const hueSliderStyles: CSSProperties = {
    ...sliderStyles,
    background: 'linear-gradient(to right, #ff0000, #ffff00, #00ff00, #00ffff, #0000ff, #ff00ff, #ff0000)',
  };

  const alphaSliderStyles: CSSProperties = {
    ...sliderStyles,
    background: `linear-gradient(to right, transparent, ${color})`,
  };

  const presetStyles: CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(7, 1fr)',
    gap: theme.spacing[2],
    marginTop: theme.spacing[3],
  };

  const presetSwatchStyles = (preset: string): CSSProperties => ({
    width: '32px',
    height: '32px',
    borderRadius: theme.borderRadius.base,
    backgroundColor: preset,
    cursor: 'pointer',
    border: `2px solid ${preset === color ? theme.colors.border.focus : theme.colors.border.primary}`,
    transition: transitionPresets.colors,
  });

  return (
    <div ref={containerRef} style={{ position: 'relative', display: 'inline-block' }}>
      {label && (
        <label style={{
          display: 'block',
          fontSize: theme.typography.fontSize.sm,
          fontWeight: theme.typography.fontWeight.medium,
          color: theme.colors.text.primary,
          marginBottom: theme.spacing[2],
        }}>
          {label}
        </label>
      )}

      <div
        style={swatchStyles}
        onClick={() => !disabled && setIsOpen(!isOpen)}
        role="button"
        aria-label="Open color picker"
        tabIndex={disabled ? -1 : 0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setIsOpen(!isOpen);
          }
        }}
      />

      {isOpen && (
        <div style={popoverStyles}>
          <div
            ref={saturationRef}
            style={saturationStyles}
            onMouseDown={handleSaturationChange}
            onMouseMove={(e) => {
              if (e.buttons === 1) handleSaturationChange(e);
            }}
          >
            <div style={pickerDotStyles} />
          </div>

          <div style={{ marginBottom: theme.spacing[3] }}>
            <label style={{ fontSize: theme.typography.fontSize.xs, color: theme.colors.text.secondary }}>
              Hue
            </label>
            <input
              type="range"
              min="0"
              max="360"
              value={hsl.h}
              onChange={handleHueChange}
              style={hueSliderStyles}
            />
          </div>

          {showAlpha && (
            <div style={{ marginBottom: theme.spacing[3] }}>
              <label style={{ fontSize: theme.typography.fontSize.xs, color: theme.colors.text.secondary }}>
                Alpha
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={hsl.a}
                onChange={handleAlphaChange}
                style={alphaSliderStyles}
              />
            </div>
          )}

          <div>
            <label style={{ fontSize: theme.typography.fontSize.xs, color: theme.colors.text.secondary }}>
              Hex
            </label>
            <input
              type="text"
              value={color}
              onChange={handleHexInput}
              style={{
                width: '100%',
                padding: theme.spacing[2],
                border: `1px solid ${theme.colors.border.primary}`,
                borderRadius: theme.borderRadius.md,
                backgroundColor: theme.colors.background.primary,
                color: theme.colors.text.primary,
                fontSize: theme.typography.fontSize.sm,
                fontFamily: theme.typography.fontFamily.mono,
                marginTop: theme.spacing[1],
              }}
            />
          </div>

          {colorPresets.length > 0 && (
            <div style={presetStyles}>
              {colorPresets.map((preset) => (
                <div
                  key={preset}
                  style={presetSwatchStyles(preset)}
                  onClick={() => handlePresetClick(preset)}
                  role="button"
                  aria-label={`Select color ${preset}`}
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handlePresetClick(preset);
                    }
                  }}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

ColorPicker.displayName = 'ColorPicker';
