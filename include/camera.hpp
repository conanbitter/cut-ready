#pragma once

#include <windows.h>
#include <gdiplus.h>

class Camera {
   public:
    Camera() : offsetx{0.0f}, offsety{0.0f}, scaleFactor{1.0f} {}
    void move(int x, int y);
    void scale(float ds, int x, int y);
    void apply(Gdiplus::Graphics &graphics);
    void apply(Gdiplus::Pen &pen);
    void setViewport(int width, int height);
    void resize(int width, int height);
    void showAll(const Gdiplus::Rect &bounds);

   private:
    float offsetx;
    float offsety;
    float scaleFactor;
    Gdiplus::Matrix matrix;
    int viewportWidth;
    int viewportHeight;

    void updateMatrix();
    void adjustOffset(int x, int y, float k);
};