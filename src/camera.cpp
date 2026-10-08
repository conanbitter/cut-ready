#include "camera.hpp"

void Camera::updateMatrix() {
    matrix.Reset();
    matrix.Translate(offsetx, offsety);
    matrix.Scale(scaleFactor, scaleFactor);
}

void Camera::move(int x, int y) {
    offsetx += x;
    offsety += y;
    updateMatrix();
}

void Camera::adjustOffset(int x, int y, float k) {
    offsetx = (float)x * (1.0f - k) + offsetx * k;
    offsety = (float)y * (1.0f - k) + offsety * k;
}

void Camera::scale(float ds, int x, int y) {
    scaleFactor *= ds;
    adjustOffset(x, y, ds);
    updateMatrix();
}

void Camera::apply(Gdiplus::Graphics &graphics) {
    graphics.SetTransform(&matrix);
}

void Camera::apply(Gdiplus::Pen &pen) {
    pen.SetWidth(2.0f / scaleFactor);
}

void Camera::setViewport(int width, int height) {
    viewportWidth = width;
    viewportHeight = height;
}
void Camera::resize(int width, int height) {
    offsetx += (float)(width - viewportWidth) / 2.0f;
    offsety += (float)(height - viewportHeight) / 2.0f;
    viewportWidth = width;
    viewportHeight = height;
    updateMatrix();
}

const int viewBorder = 20;

void Camera::showAll(const Gdiplus::Rect &bounds) {
    float scale = (float)(viewportWidth - viewBorder * 2) / (float)bounds.Width;
    float scaley = (float)(viewportHeight - viewBorder * 2) / (float)bounds.Height;
    if (scaley < scale) {
        scale = scaley;
    }
    scaleFactor = scale;
    offsetx = (float)viewportWidth / 2.0f - ((float)bounds.X + (float)bounds.Width / 2.0f) * scale;
    offsety = (float)viewportHeight / 2.0f - ((float)bounds.Y + (float)bounds.Height / 2.0f) * scale;
    updateMatrix();
}