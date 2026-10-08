#pragma once

#include <windows.h>
#include <gdiplus.h>
#include <map>

#include "camera.hpp"

class Drawing {
    friend class Plotter;

   public:
    Drawing() : pen(Gdiplus::Color::Black, 1.0f) {}

   private:
    Gdiplus::Pen pen;
    Gdiplus::GraphicsPath path;
};

class Plotter {
   public:
    Plotter()
        : backgroundBrush(backgroundColor),
          fillBrush(fillColor),
          pen(lineColor, lineWidth) {}
    void draw(Gdiplus::Graphics& gfx, Gdiplus::Rect& area);
    Camera& getCamera() { return camera; }
    void resetView() { camera.showAll(bounds); }
    void updateBounds();
    void clear();
    Gdiplus::GraphicsPath& getPath(int index);
    Gdiplus::Pen pen;
    Gdiplus::GraphicsPath* path = nullptr;

   private:
    Camera camera;

    Gdiplus::SolidBrush backgroundBrush;
    Gdiplus::SolidBrush fillBrush;

    std::map<int, Drawing> drawings;
    Gdiplus::Rect bounds;

    static const Gdiplus::Color backgroundColor;
    static const Gdiplus::Color lineColor;
    static const Gdiplus::Color fillColor;
    static const float lineWidth;
};