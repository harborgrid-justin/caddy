import React from 'react';
import { CameraState, RenderStatistics } from './ViewportManager';
export declare enum OverlayType {
    Text = "text",
    Measurement = "measurement",
    Dimension = "dimension",
    Annotation = "annotation",
    Cursor = "cursor",
    Icon = "icon",
    Line = "line",
    Rectangle = "rectangle",
    Circle = "circle"
}
export declare enum AnchorPosition {
    TopLeft = "top-left",
    TopCenter = "top-center",
    TopRight = "top-right",
    MiddleLeft = "middle-left",
    Center = "center",
    MiddleRight = "middle-right",
    BottomLeft = "bottom-left",
    BottomCenter = "bottom-center",
    BottomRight = "bottom-right"
}
export interface OverlayElement {
    id: string;
    type: OverlayType;
    visible: boolean;
    position: [number, number];
    anchor?: AnchorPosition;
    opacity?: number;
    zIndex?: number;
}
export interface TextOverlay extends OverlayElement {
    type: OverlayType.Text;
    text: string;
    fontSize?: number;
    fontFamily?: string;
    color?: string;
    backgroundColor?: string;
    padding?: number;
    borderRadius?: number;
}
export interface MeasurementOverlay extends OverlayElement {
    type: OverlayType.Measurement;
    startPoint: [number, number, number];
    endPoint: [number, number, number];
    value: number;
    unit: string;
    label?: string;
    color?: string;
    lineWidth?: number;
    showEndpoints?: boolean;
}
export interface DimensionOverlay extends OverlayElement {
    type: OverlayType.Dimension;
    points: Array<[number, number, number]>;
    value: number;
    unit: string;
    label?: string;
    color?: string;
    offset?: number;
}
export interface AnnotationOverlay extends OverlayElement {
    type: OverlayType.Annotation;
    worldPosition: [number, number, number];
    title: string;
    description?: string;
    icon?: string;
    color?: string;
    size?: number;
}
export interface CursorOverlay extends OverlayElement {
    type: OverlayType.Cursor;
    cursorType: 'crosshair' | 'circle' | 'square' | 'custom';
    size?: number;
    color?: string;
}
export interface LineOverlay extends OverlayElement {
    type: OverlayType.Line;
    endPosition: [number, number];
    color?: string;
    width?: number;
    style?: 'solid' | 'dashed' | 'dotted';
}
export interface RectangleOverlay extends OverlayElement {
    type: OverlayType.Rectangle;
    width: number;
    height: number;
    fillColor?: string;
    strokeColor?: string;
    strokeWidth?: number;
}
export interface CircleOverlay extends OverlayElement {
    type: OverlayType.Circle;
    radius: number;
    fillColor?: string;
    strokeColor?: string;
    strokeWidth?: number;
}
export type AnyOverlay = TextOverlay | MeasurementOverlay | DimensionOverlay | AnnotationOverlay | CursorOverlay | LineOverlay | RectangleOverlay | CircleOverlay;
export interface ViewportOverlayProps {
    camera: CameraState;
    width: number;
    height: number;
    overlays?: AnyOverlay[];
    showFPS?: boolean;
    showStats?: boolean;
    stats?: RenderStatistics;
    showCoordinates?: boolean;
    cursorPosition?: [number, number, number];
    className?: string;
    style?: React.CSSProperties;
}
export declare const ViewportOverlay: React.FC<ViewportOverlayProps>;
export default ViewportOverlay;
//# sourceMappingURL=ViewportOverlay.d.ts.map