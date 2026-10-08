#include "plotter.hpp"

const Gdiplus::Color Plotter::backgroundColor = Gdiplus::Color(255, 236, 239, 241);
const Gdiplus::Color Plotter::lineColor = Gdiplus::Color(255, 30, 136, 229);
const Gdiplus::Color Plotter::fillColor = Gdiplus::Color(128, 102, 187, 106);
const float Plotter::lineWidth = 2.0f;

static const Gdiplus::Color defaultColors[] = {
    Gdiplus::Color(255, 0, 0, 0),
    Gdiplus::Color(255, 255, 0, 0),
    Gdiplus::Color(255, 255, 255, 0),
    Gdiplus::Color(255, 0, 255, 0),
    Gdiplus::Color(255, 0, 255, 255),
    Gdiplus::Color(255, 0, 0, 255),
    Gdiplus::Color(255, 255, 0, 255),
    Gdiplus::Color(255, 0, 0, 0),
    Gdiplus::Color(255, 65, 65, 65),
    Gdiplus::Color(255, 128, 128, 128),
    Gdiplus::Color(255, 255, 0, 0),
    Gdiplus::Color(255, 255, 170, 170),
    Gdiplus::Color(255, 189, 0, 0),
    Gdiplus::Color(255, 189, 126, 126),
    Gdiplus::Color(255, 129, 0, 0),
    Gdiplus::Color(255, 129, 86, 86),
    Gdiplus::Color(255, 104, 0, 0),
    Gdiplus::Color(255, 104, 69, 69),
    Gdiplus::Color(255, 79, 0, 0),
    Gdiplus::Color(255, 79, 53, 53),
    Gdiplus::Color(255, 255, 63, 0),
    Gdiplus::Color(255, 255, 191, 170),
    Gdiplus::Color(255, 189, 46, 0),
    Gdiplus::Color(255, 189, 141, 126),
    Gdiplus::Color(255, 129, 31, 0),
    Gdiplus::Color(255, 129, 96, 86),
    Gdiplus::Color(255, 104, 25, 0),
    Gdiplus::Color(255, 104, 78, 69),
    Gdiplus::Color(255, 79, 19, 0),
    Gdiplus::Color(255, 79, 59, 53),
    Gdiplus::Color(255, 255, 127, 0),
    Gdiplus::Color(255, 255, 212, 170),
    Gdiplus::Color(255, 189, 94, 0),
    Gdiplus::Color(255, 189, 157, 126),
    Gdiplus::Color(255, 129, 64, 0),
    Gdiplus::Color(255, 129, 107, 86),
    Gdiplus::Color(255, 104, 52, 0),
    Gdiplus::Color(255, 104, 86, 69),
    Gdiplus::Color(255, 79, 39, 0),
    Gdiplus::Color(255, 79, 66, 53),
    Gdiplus::Color(255, 255, 191, 0),
    Gdiplus::Color(255, 255, 234, 170),
    Gdiplus::Color(255, 189, 141, 0),
    Gdiplus::Color(255, 189, 173, 126),
    Gdiplus::Color(255, 129, 96, 0),
    Gdiplus::Color(255, 129, 118, 86),
    Gdiplus::Color(255, 104, 78, 0),
    Gdiplus::Color(255, 104, 95, 69),
    Gdiplus::Color(255, 79, 59, 0),
    Gdiplus::Color(255, 79, 73, 53),
    Gdiplus::Color(255, 255, 255, 0),
    Gdiplus::Color(255, 255, 255, 170),
    Gdiplus::Color(255, 189, 189, 0),
    Gdiplus::Color(255, 189, 189, 126),
    Gdiplus::Color(255, 129, 129, 0),
    Gdiplus::Color(255, 129, 129, 86),
    Gdiplus::Color(255, 104, 104, 0),
    Gdiplus::Color(255, 104, 104, 69),
    Gdiplus::Color(255, 79, 79, 0),
    Gdiplus::Color(255, 79, 79, 53),
    Gdiplus::Color(255, 191, 255, 0),
    Gdiplus::Color(255, 234, 255, 170),
    Gdiplus::Color(255, 141, 189, 0),
    Gdiplus::Color(255, 173, 189, 126),
    Gdiplus::Color(255, 96, 129, 0),
    Gdiplus::Color(255, 118, 129, 86),
    Gdiplus::Color(255, 78, 104, 0),
    Gdiplus::Color(255, 95, 104, 69),
    Gdiplus::Color(255, 59, 79, 0),
    Gdiplus::Color(255, 73, 79, 53),
    Gdiplus::Color(255, 127, 255, 0),
    Gdiplus::Color(255, 212, 255, 170),
    Gdiplus::Color(255, 94, 189, 0),
    Gdiplus::Color(255, 157, 189, 126),
    Gdiplus::Color(255, 64, 129, 0),
    Gdiplus::Color(255, 107, 129, 86),
    Gdiplus::Color(255, 52, 104, 0),
    Gdiplus::Color(255, 86, 104, 69),
    Gdiplus::Color(255, 39, 79, 0),
    Gdiplus::Color(255, 66, 79, 53),
    Gdiplus::Color(255, 63, 255, 0),
    Gdiplus::Color(255, 191, 255, 170),
    Gdiplus::Color(255, 46, 189, 0),
    Gdiplus::Color(255, 141, 189, 126),
    Gdiplus::Color(255, 31, 129, 0),
    Gdiplus::Color(255, 96, 129, 86),
    Gdiplus::Color(255, 25, 104, 0),
    Gdiplus::Color(255, 78, 104, 69),
    Gdiplus::Color(255, 19, 79, 0),
    Gdiplus::Color(255, 59, 79, 53),
    Gdiplus::Color(255, 0, 255, 0),
    Gdiplus::Color(255, 170, 255, 170),
    Gdiplus::Color(255, 0, 189, 0),
    Gdiplus::Color(255, 126, 189, 126),
    Gdiplus::Color(255, 0, 129, 0),
    Gdiplus::Color(255, 86, 129, 86),
    Gdiplus::Color(255, 0, 104, 0),
    Gdiplus::Color(255, 69, 104, 69),
    Gdiplus::Color(255, 0, 79, 0),
    Gdiplus::Color(255, 53, 79, 53),
    Gdiplus::Color(255, 0, 255, 63),
    Gdiplus::Color(255, 170, 255, 191),
    Gdiplus::Color(255, 0, 189, 46),
    Gdiplus::Color(255, 126, 189, 141),
    Gdiplus::Color(255, 0, 129, 31),
    Gdiplus::Color(255, 86, 129, 96),
    Gdiplus::Color(255, 0, 104, 25),
    Gdiplus::Color(255, 69, 104, 78),
    Gdiplus::Color(255, 0, 79, 19),
    Gdiplus::Color(255, 53, 79, 59),
    Gdiplus::Color(255, 0, 255, 127),
    Gdiplus::Color(255, 170, 255, 212),
    Gdiplus::Color(255, 0, 189, 94),
    Gdiplus::Color(255, 126, 189, 157),
    Gdiplus::Color(255, 0, 129, 64),
    Gdiplus::Color(255, 86, 129, 107),
    Gdiplus::Color(255, 0, 104, 52),
    Gdiplus::Color(255, 69, 104, 86),
    Gdiplus::Color(255, 0, 79, 39),
    Gdiplus::Color(255, 53, 79, 66),
    Gdiplus::Color(255, 0, 255, 191),
    Gdiplus::Color(255, 170, 255, 234),
    Gdiplus::Color(255, 0, 189, 141),
    Gdiplus::Color(255, 126, 189, 173),
    Gdiplus::Color(255, 0, 129, 96),
    Gdiplus::Color(255, 86, 129, 118),
    Gdiplus::Color(255, 0, 104, 78),
    Gdiplus::Color(255, 69, 104, 95),
    Gdiplus::Color(255, 0, 79, 59),
    Gdiplus::Color(255, 53, 79, 73),
    Gdiplus::Color(255, 0, 255, 255),
    Gdiplus::Color(255, 170, 255, 255),
    Gdiplus::Color(255, 0, 189, 189),
    Gdiplus::Color(255, 126, 189, 189),
    Gdiplus::Color(255, 0, 129, 129),
    Gdiplus::Color(255, 86, 129, 129),
    Gdiplus::Color(255, 0, 104, 104),
    Gdiplus::Color(255, 69, 104, 104),
    Gdiplus::Color(255, 0, 79, 79),
    Gdiplus::Color(255, 53, 79, 79),
    Gdiplus::Color(255, 0, 191, 255),
    Gdiplus::Color(255, 170, 234, 255),
    Gdiplus::Color(255, 0, 141, 189),
    Gdiplus::Color(255, 126, 173, 189),
    Gdiplus::Color(255, 0, 96, 129),
    Gdiplus::Color(255, 86, 118, 129),
    Gdiplus::Color(255, 0, 78, 104),
    Gdiplus::Color(255, 69, 95, 104),
    Gdiplus::Color(255, 0, 59, 79),
    Gdiplus::Color(255, 53, 73, 79),
    Gdiplus::Color(255, 0, 127, 255),
    Gdiplus::Color(255, 170, 212, 255),
    Gdiplus::Color(255, 0, 94, 189),
    Gdiplus::Color(255, 126, 157, 189),
    Gdiplus::Color(255, 0, 64, 129),
    Gdiplus::Color(255, 86, 107, 129),
    Gdiplus::Color(255, 0, 52, 104),
    Gdiplus::Color(255, 69, 86, 104),
    Gdiplus::Color(255, 0, 39, 79),
    Gdiplus::Color(255, 53, 66, 79),
    Gdiplus::Color(255, 0, 63, 255),
    Gdiplus::Color(255, 170, 191, 255),
    Gdiplus::Color(255, 0, 46, 189),
    Gdiplus::Color(255, 126, 141, 189),
    Gdiplus::Color(255, 0, 31, 129),
    Gdiplus::Color(255, 86, 96, 129),
    Gdiplus::Color(255, 0, 25, 104),
    Gdiplus::Color(255, 69, 78, 104),
    Gdiplus::Color(255, 0, 19, 79),
    Gdiplus::Color(255, 53, 59, 79),
    Gdiplus::Color(255, 0, 0, 255),
    Gdiplus::Color(255, 170, 170, 255),
    Gdiplus::Color(255, 0, 0, 189),
    Gdiplus::Color(255, 126, 126, 189),
    Gdiplus::Color(255, 0, 0, 129),
    Gdiplus::Color(255, 86, 86, 129),
    Gdiplus::Color(255, 0, 0, 104),
    Gdiplus::Color(255, 69, 69, 104),
    Gdiplus::Color(255, 0, 0, 79),
    Gdiplus::Color(255, 53, 53, 79),
    Gdiplus::Color(255, 63, 0, 255),
    Gdiplus::Color(255, 191, 170, 255),
    Gdiplus::Color(255, 46, 0, 189),
    Gdiplus::Color(255, 141, 126, 189),
    Gdiplus::Color(255, 31, 0, 129),
    Gdiplus::Color(255, 96, 86, 129),
    Gdiplus::Color(255, 25, 0, 104),
    Gdiplus::Color(255, 78, 69, 104),
    Gdiplus::Color(255, 19, 0, 79),
    Gdiplus::Color(255, 59, 53, 79),
    Gdiplus::Color(255, 127, 0, 255),
    Gdiplus::Color(255, 212, 170, 255),
    Gdiplus::Color(255, 94, 0, 189),
    Gdiplus::Color(255, 157, 126, 189),
    Gdiplus::Color(255, 64, 0, 129),
    Gdiplus::Color(255, 107, 86, 129),
    Gdiplus::Color(255, 52, 0, 104),
    Gdiplus::Color(255, 86, 69, 104),
    Gdiplus::Color(255, 39, 0, 79),
    Gdiplus::Color(255, 66, 53, 79),
    Gdiplus::Color(255, 191, 0, 255),
    Gdiplus::Color(255, 234, 170, 255),
    Gdiplus::Color(255, 141, 0, 189),
    Gdiplus::Color(255, 173, 126, 189),
    Gdiplus::Color(255, 96, 0, 129),
    Gdiplus::Color(255, 118, 86, 129),
    Gdiplus::Color(255, 78, 0, 104),
    Gdiplus::Color(255, 95, 69, 104),
    Gdiplus::Color(255, 59, 0, 79),
    Gdiplus::Color(255, 73, 53, 79),
    Gdiplus::Color(255, 255, 0, 255),
    Gdiplus::Color(255, 255, 170, 255),
    Gdiplus::Color(255, 189, 0, 189),
    Gdiplus::Color(255, 189, 126, 189),
    Gdiplus::Color(255, 129, 0, 129),
    Gdiplus::Color(255, 129, 86, 129),
    Gdiplus::Color(255, 104, 0, 104),
    Gdiplus::Color(255, 104, 69, 104),
    Gdiplus::Color(255, 79, 0, 79),
    Gdiplus::Color(255, 79, 53, 79),
    Gdiplus::Color(255, 255, 0, 191),
    Gdiplus::Color(255, 255, 170, 234),
    Gdiplus::Color(255, 189, 0, 141),
    Gdiplus::Color(255, 189, 126, 173),
    Gdiplus::Color(255, 129, 0, 96),
    Gdiplus::Color(255, 129, 86, 118),
    Gdiplus::Color(255, 104, 0, 78),
    Gdiplus::Color(255, 104, 69, 95),
    Gdiplus::Color(255, 79, 0, 59),
    Gdiplus::Color(255, 79, 53, 73),
    Gdiplus::Color(255, 255, 0, 127),
    Gdiplus::Color(255, 255, 170, 212),
    Gdiplus::Color(255, 189, 0, 94),
    Gdiplus::Color(255, 189, 126, 157),
    Gdiplus::Color(255, 129, 0, 64),
    Gdiplus::Color(255, 129, 86, 107),
    Gdiplus::Color(255, 104, 0, 52),
    Gdiplus::Color(255, 104, 69, 86),
    Gdiplus::Color(255, 79, 0, 39),
    Gdiplus::Color(255, 79, 53, 66),
    Gdiplus::Color(255, 255, 0, 63),
    Gdiplus::Color(255, 255, 170, 191),
    Gdiplus::Color(255, 189, 0, 46),
    Gdiplus::Color(255, 189, 126, 141),
    Gdiplus::Color(255, 129, 0, 31),
    Gdiplus::Color(255, 129, 86, 96),
    Gdiplus::Color(255, 104, 0, 25),
    Gdiplus::Color(255, 104, 69, 78),
    Gdiplus::Color(255, 79, 0, 19),
    Gdiplus::Color(255, 79, 53, 59),
    Gdiplus::Color(255, 51, 51, 51),
    Gdiplus::Color(255, 80, 80, 80),
    Gdiplus::Color(255, 105, 105, 105),
    Gdiplus::Color(255, 130, 130, 130),
    Gdiplus::Color(255, 190, 190, 190),
    Gdiplus::Color(255, 255, 255, 255),
};

void Plotter::draw(Gdiplus::Graphics& gfx, Gdiplus::Rect& area) {
    gfx.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);
    pen.SetLineJoin(Gdiplus::LineJoinRound);

    gfx.FillRectangle(&backgroundBrush, area);
    // Gdiplus::Pen pen(lineColor, lineWidth);
    // pen.SetColor(lineColor);
    camera.apply(gfx);
    for (auto& drawing : drawings) {
        camera.apply(drawing.second.pen);
        gfx.DrawPath(&drawing.second.pen, &drawing.second.path);
    }
    // gfx.DrawPath(&pen, path);

    /*gfx.DrawLine(&pen, 100, 100, 300, 200);
    Gdiplus::Point p[] = {
        Gdiplus::Point(10, 100),   // start point of first spline
        Gdiplus::Point(75, 10),    // first control point of first spline
        Gdiplus::Point(80, 50),    // second control point of first spline
        Gdiplus::Point(100, 150),  // end point of first spline and
                                   // start point of second spline
        Gdiplus::Point(125, 80),   // first control point of second spline
        Gdiplus::Point(175, 200),  // second control point of second spline
        Gdiplus::Point(200, 80)};  // end point of second spline
    gfx.DrawBeziers(&pen, p, 7);

    Gdiplus::GraphicsPath path;
    path.AddLine(350, 500, 550, 100);
    path.AddLine(550, 100, 500, 300);
    path.AddBezier(500, 300, 600, 400, 600, 500, 500, 550);
    path.CloseFigure();
    gfx.FillPath(&fillBrush, &path);
    gfx.DrawPath(&pen, &path);*/
}

Gdiplus::GraphicsPath& Plotter::getPath(int index) {
    if (drawings.find(index) == drawings.end()) {
        drawings[index].pen.SetColor(index < 256 ? defaultColors[index] : Gdiplus::Color::Black);
        drawings[index].pen.SetLineJoin(Gdiplus::LineJoinRound);
        drawings[index].pen.SetLineCap(Gdiplus::LineCapRound, Gdiplus::LineCapRound, Gdiplus::DashCapRound);
    }
    return drawings[index].path;
}

void Plotter::updateBounds() {
    int x1 = 0, x2 = 0, y1 = 0, y2 = 0;
    for (auto& drawing : drawings) {
        Gdiplus::Rect rect;
        drawing.second.path.GetBounds(&rect, nullptr, &drawing.second.pen);
        int x1i = rect.X;
        int y1i = rect.Y;
        int x2i = rect.X + rect.Width;
        int y2i = rect.Y + rect.Height;
        if (x1i < x1) x1 = x1i;
        if (x2i > x2) x2 = x2i;
        if (y1i < y1) y1 = y1i;
        if (y2i > y2) y2 = y2i;
    }
    bounds.X = x1;
    bounds.Y = y1;
    bounds.Width = x2 - x1;
    bounds.Height = y2 - y1;
}

void Plotter::clear() {
    drawings.clear();
}