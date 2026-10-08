#include "dxf.hpp"
#include <windows.h>
#include <vector>
#include "tinysplinecxx.h"

void readFile(const wchar_t* filename, Plotter& plotter) {
    dxfRW file(filename);
    DXFReader reader(plotter);
    file.read(&reader, false);
    plotter.updateBounds();
}

static void consoleOut(const std::string message) {
    HANDLE stdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (stdOut != NULL && stdOut != INVALID_HANDLE_VALUE) {
        DWORD written = 0;
        WriteConsoleA(stdOut, message.c_str(), message.length(), &written, NULL);
        WriteConsoleA(stdOut, "\n", 1, &written, NULL);
    }
}

void DXFReader::addHeader(const DRW_Header* data) {
    // consoleOut("addHeader");
}

void DXFReader::addLType(const DRW_LType& data) {
}

void DXFReader::addLayer(const DRW_Layer& data) {
    // consoleOut("addLayer");
}

void DXFReader::addDimStyle(const DRW_Dimstyle& data) {
    // consoleOut("addDimStyle");
}

void DXFReader::addVport(const DRW_Vport& data) {
    // consoleOut("addVport");
}

void DXFReader::addTextStyle(const DRW_Textstyle& data) {
    // consoleOut("addTextStyle");
}

void DXFReader::addAppId(const DRW_AppId& data) {
    // consoleOut("addAppId");
}

void DXFReader::addBlock(const DRW_Block& data) {
    // consoleOut("addBlock");
}

void DXFReader::setBlock(const int handle) {
    consoleOut("setBlock");
}

void DXFReader::endBlock() {
    // consoleOut("endBlock");
}

void DXFReader::addPoint(const DRW_Point& data) {
    consoleOut("addPoint");
}

void DXFReader::addLine(const DRW_Line& data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data.color);
    path.StartFigure();
    path.AddLine(
        (Gdiplus::REAL)data.basePoint.x,
        -(Gdiplus::REAL)data.basePoint.y,
        (Gdiplus::REAL)data.secPoint.x,
        -(Gdiplus::REAL)data.secPoint.y);
    // consoleOut("addLine");
}

void DXFReader::addRay(const DRW_Ray& data) {
    consoleOut("addRay");
}

void DXFReader::addXline(const DRW_Xline& data) {
    consoleOut("addXline");
}

void DXFReader::addArc(const DRW_Arc& data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data.color);
    path.StartFigure();
    if (data.staangle < data.endangle) {
        path.AddArc(
            (Gdiplus::REAL)(data.basePoint.x - data.radious),
            -(Gdiplus::REAL)(data.basePoint.y + data.radious),
            (Gdiplus::REAL)(data.radious * 2),
            (Gdiplus::REAL)(data.radious * 2),
            -(Gdiplus::REAL)(data.staangle / M_PI * 180.0),
            -(Gdiplus::REAL)((data.endangle - data.staangle) / M_PI * 180.0));
    } else {
        path.AddArc(
            (Gdiplus::REAL)(data.basePoint.x - data.radious),
            -(Gdiplus::REAL)(data.basePoint.y + data.radious),
            (Gdiplus::REAL)(data.radious * 2),
            (Gdiplus::REAL)(data.radious * 2),
            -(Gdiplus::REAL)(data.staangle / M_PI * 180.0),
            -(Gdiplus::REAL)((data.endangle + M_PIx2 - data.staangle) / M_PI * 180.0));
    }
    // consoleOut("addArc");
}

void DXFReader::addCircle(const DRW_Circle& data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data.color);
    path.StartFigure();
    path.AddEllipse(
        (Gdiplus::REAL)(data.basePoint.x - data.radious),
        -(Gdiplus::REAL)(data.basePoint.y + data.radious),
        (Gdiplus::REAL)(data.radious * 2),
        (Gdiplus::REAL)(data.radious * 2));
    // consoleOut("addCircle");
}

static void lineLengthAngle(const DRW_Line& line, float& length, float& angle) {
    length = sqrt(line.secPoint.x * line.secPoint.x + line.secPoint.y * line.secPoint.y);
    angle = acos(line.secPoint.x / length) / M_PI * 180.0;
    if (line.secPoint.y > 0) {
        angle = -angle;
    }
}

static const float epsilon = 0.0001;

static bool isFull(const DRW_Ellipse& data) {
    return (abs(data.staparam) < epsilon) && (abs(data.endparam - M_PIx2) < epsilon);
}

void DXFReader::addEllipse(const DRW_Ellipse& data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data.color);
    Gdiplus::GraphicsPath ellipsePath;
    float mainRadius;
    float angle;
    lineLengthAngle(data, mainRadius, angle);
    float secRaduis = mainRadius * data.ratio;
    bool fullArc = isFull(data);
    if (fullArc) {
        ellipsePath.AddEllipse(
            (Gdiplus::REAL)(-mainRadius),
            (Gdiplus::REAL)(-secRaduis),
            (Gdiplus::REAL)(mainRadius * 2),
            (Gdiplus::REAL)(secRaduis * 2));
    } else {
        ellipsePath.AddArc(
            (Gdiplus::REAL)(-mainRadius),
            (Gdiplus::REAL)(-mainRadius),
            (Gdiplus::REAL)(mainRadius * 2),
            (Gdiplus::REAL)(mainRadius * 2),
            -(Gdiplus::REAL)(data.staparam / M_PI * 180.0),
            -(Gdiplus::REAL)((data.endparam - data.staparam) / M_PI * 180.0));
    }
    Gdiplus::Matrix transform;
    transform.Translate(data.basePoint.x, -data.basePoint.y);
    transform.Rotate(angle);
    if (!fullArc) {
        transform.Scale(1.0f, data.ratio);
    }
    ellipsePath.Transform(&transform);
    path.AddPath(&ellipsePath, false);
    // consoleOut("addEllipse");
}

void DXFReader::addLWPolyline(const DRW_LWPolyline& data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data.color);
    std::vector<Gdiplus::PointF> points;
    points.reserve(data.vertexnum);
    for (auto point : data.vertlist) {
        // point->bulge
        points.push_back(Gdiplus::PointF(point->x, -point->y));
    }
    path.StartFigure();
    if (data.flags == 1) {
        path.AddPolygon(points.data(), points.size());
    } else {
        path.AddLines(points.data(), points.size());
    }
    // consoleOut("addLWPolyline");
}

void DXFReader::addPolyline(const DRW_Polyline& data) {
    consoleOut("addPolyline");
}

void DXFReader::addSpline(const DRW_Spline* data) {
    Gdiplus::GraphicsPath& path = plotter.getPath(data->color);
    /*std::vector<Gdiplus::PointF> points;
    points.reserve(data->nfit);
    for (auto point : data->fitlist) {
        points.push_back(Gdiplus::PointF(point->x, -point->y));
    }
    path->StartFigure();
    if (data->flags == 1) {
        path->AddPolygon(points.data(), points.size());
    } else {
        path->AddLines(points.data(), points.size());
    }

    points.clear();
    points.reserve(data->ncontrol);
    for (auto point : data->controllist) {
        points.push_back(Gdiplus::PointF(point->x, -point->y));
    }
    path->StartFigure();
    path->AddBeziers(points.data(), points.size());
    path->AddLines(points.data(), points.size());*/
    tinyspline::BSpline bspline;
    if (data->flags & 4) {
        bspline = tinyspline::BSpline(data->ncontrol, 3, data->degree);
        std::vector<tinyspline::real> ctrlp = bspline.controlPoints();
        for (int i = 0; i < data->ncontrol; i++) {
            ctrlp[i * 3] = data->controllist[i]->x * data->weightlist[i];
            ctrlp[i * 3 + 1] = -data->controllist[i]->y * data->weightlist[i];
            ctrlp[i * 3 + 2] = data->weightlist[i];
        }
        bspline.setControlPoints(ctrlp);
    } else {
        bspline = tinyspline::BSpline(data->ncontrol, 2, data->degree);
        std::vector<tinyspline::real> ctrlp = bspline.controlPoints();
        for (int i = 0; i < data->ncontrol; i++) {
            ctrlp[i * 2] = data->controllist[i]->x;
            ctrlp[i * 2 + 1] = -data->controllist[i]->y;
        }
        bspline.setControlPoints(ctrlp);
    }
    if (data->nknots > 0) {
        std::vector<tinyspline::real> knots;
        for (auto kn : data->knotslist) {
            knots.push_back(kn);
        }
        try {
            bspline.setKnots(knots);
        } catch (const std::exception& e) {
        }
    }
    auto poly = bspline.sample(100);
    std::vector<Gdiplus::PointF> points;
    if (data->flags & 4) {
        points.reserve(poly.size() / 3);
        for (int i = 0; i < poly.size(); i += 3) {
            points.push_back(Gdiplus::PointF(poly[i] / poly[i + 2], poly[i + 1] / poly[i + 2]));
        }
    } else {
        points.reserve(poly.size() / 2);
        for (int i = 0; i < poly.size(); i += 2) {
            points.push_back(Gdiplus::PointF(poly[i], poly[i + 1]));
        }
    }
    path.StartFigure();
    path.AddLines(points.data(), points.size());
    bspline = bspline.toBeziers();
    if (bspline.degree() == 2) {
        bspline = bspline.elevateDegree(1);
    }
    /*
    // std::vector<Gdiplus::PointF> points;
    points.clear();
    if (data->flags & 4) {
        points.reserve(bspline.numControlPoints());
        for (int i = 0; i < bspline.numControlPoints(); i++) {
            if (i > 0 && i % 4 == 0) continue;
            Gdiplus::REAL w = bspline.controlPoints()[i * 3 + 2];
            Gdiplus::REAL x = bspline.controlPoints()[i * 3] / w;
            Gdiplus::REAL y = bspline.controlPoints()[i * 3 + 1] / w;
            points.push_back(Gdiplus::PointF(x, y));
        }
    } else {
        points.reserve(bspline.numControlPoints());
        for (int i = 0; i < bspline.numControlPoints(); i++) {
            if (i > 0 && i % 4 == 0) continue;
            Gdiplus::REAL x = bspline.controlPoints()[i * 2];
            Gdiplus::REAL y = bspline.controlPoints()[i * 2 + 1];
            points.push_back(Gdiplus::PointF(x, y));
        }
    }

    path->StartFigure();
    path->AddBeziers(points.data(), points.size());*/

    /*consoleOut("addSpline");
    if (data->flags & 1) consoleOut("    - closed");
    if (data->flags & 2) consoleOut("    - periodic");
    if (data->flags & 4) consoleOut("    - rational");
    if (data->flags & 8) consoleOut("    - planar");
    if (data->flags & 16) consoleOut("    - linear");*/
}

void DXFReader::addKnot(const DRW_Entity& data) {
    consoleOut("addKnot");
}

void DXFReader::addInsert(const DRW_Insert& data) {
    // consoleOut("addInsert");
}

void DXFReader::addTrace(const DRW_Trace& data) {
    consoleOut("addTrace");
}

void DXFReader::add3dFace(const DRW_3Dface& data) {
    consoleOut("add3dFace");
}

void DXFReader::addSolid(const DRW_Solid& data) {
    consoleOut("addSolid");
}

void DXFReader::addMText(const DRW_MText& data) {
    consoleOut("addMText");
}

void DXFReader::addText(const DRW_Text& data) {
    consoleOut("addText");
}

void DXFReader::addDimAlign(const DRW_DimAligned* data) {
    consoleOut("addDimAlign");
}

void DXFReader::addDimLinear(const DRW_DimLinear* data) {
    consoleOut("addDimLinear");
}

void DXFReader::addDimRadial(const DRW_DimRadial* data) {
    consoleOut("addDimRadial");
}

void DXFReader::addDimDiametric(const DRW_DimDiametric* data) {
    consoleOut("addDimDiametric");
}

void DXFReader::addDimAngular(const DRW_DimAngular* data) {
    consoleOut("addDimAngular");
}

void DXFReader::addDimAngular3P(const DRW_DimAngular3p* data) {
    consoleOut("addDimAngular3P");
}

void DXFReader::addDimOrdinate(const DRW_DimOrdinate* data) {
    consoleOut("addDimOrdinate");
}

void DXFReader::addLeader(const DRW_Leader* data) {
    consoleOut("addLeader");
}

void DXFReader::addHatch(const DRW_Hatch* data) {
    consoleOut("addHatch");
}

void DXFReader::addViewport(const DRW_Viewport& data) {
    consoleOut("addViewport");
}

void DXFReader::addImage(const DRW_Image* data) {
    consoleOut("addImage");
}

void DXFReader::linkImage(const DRW_ImageDef* data) {
    consoleOut("linkImage");
}

void DXFReader::addComment(const char* comment) {
    consoleOut("addComment");
}

void DXFReader::addPlotSettings(const DRW_PlotSettings* data) {
    consoleOut("addPlotSettings");
}