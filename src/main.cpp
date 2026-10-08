#include <windows.h>
#include <gdiplus.h>
#include <windowsx.h>
#include <iostream>
#include <sstream>
#include <Shlwapi.h>

#include "winapp.hpp"
#include "camera.hpp"
#include "plotter.hpp"
#include "dxf.hpp"

const int windowWidth = 800;
const int windowHeight = 600;

static void consoleOut(const std::string message) {
    HANDLE stdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (stdOut != NULL && stdOut != INVALID_HANDLE_VALUE) {
        DWORD written = 0;
        WriteConsoleA(stdOut, message.c_str(), message.length(), &written, NULL);
    }
}

class CheckerApp : public Application {
   public:
    using Application::Application;

    static bool isFile(const std::wstring& filename) {
        DWORD dwAttrib = GetFileAttributes(filename.c_str());
        return dwAttrib != INVALID_FILE_ATTRIBUTES && !(dwAttrib & FILE_ATTRIBUTE_DIRECTORY);
    }

    bool openFile(const std::wstring& filename) {
        if (!filename.empty() && isFile(filename)) {
            std::wstring filestring(PathFindFileName(filename.c_str()));
            filestring += L" - Cut Checker 0.1";
            wnd.setTitle(filestring.c_str());
            plotter.clear();
            readFile(filename.c_str(), plotter);
            plotter.resetView();
            wnd.update();
            return true;
        }
        return false;
    }

    void onInit() override {
        // AllocConsole();
        plotter.getCamera().setViewport(windowWidth, windowHeight);
        if (!openFile(argument)) {
            openFile(L"../../tests/test.dxf");
        }
        wnd.setAcceptDropFiles(true);
    }

    void onMouseDown(int button, int x, int y) override {
        if (button == BUTTON_LEFT) {
            oldX = x;
            oldY = y;
            isDragging = true;
            wnd.setMouseCapture(true);
        }
    }

    void onMouseMove(int buttons, int x, int y) override {
        if (isDragging) {
            if (!(buttons & BUTTON_LEFT)) {
                isDragging = false;
                return;
            }
            // std::stringstream strm;
            // strm << "Move x: " << x << "   y: " << y << "\n";
            // consoleOut(strm.str());
            plotter.getCamera().move(x - oldX, y - oldY);
            oldX = x;
            oldY = y;
            wnd.update();
        }
    }

    void onMouseUp(int button, int x, int y) override {
        if (button == BUTTON_LEFT) {
            isDragging = false;
            wnd.setMouseCapture(false);
        }
    }

    void onMouseWheel(int direction, int x, int y) override {
        // std::stringstream strm;
        // strm << "Wheel x: " << x << "   y: " << y << "\n";
        // consoleOut(strm.str());
        if (direction < 0) {
            plotter.getCamera().scale(0.9, x, y);
        } else {
            plotter.getCamera().scale(1 / 0.9, x, y);
        }
        wnd.update();
    }

    virtual void onDraw(Gdiplus::Graphics& gfx, Gdiplus::Rect& area) {
        plotter.draw(gfx, area);
    }

    void onResize(int newWidth, int newHeight) override {
        plotter.getCamera().resize(newWidth, newHeight);
        // std::stringstream strm;
        // strm << "Resize w: " << newWidth << "   h: " << newHeight << "\n";
        // consoleOut(strm.str());
        //  updateWindow();
    }

    void onFilesDrop(std::vector<std::wstring> filelist) override {
        std::wstring file;
        bool fileFound = false;
        for (auto fileitem : filelist) {
            if (isFile(fileitem)) {
                file = fileitem;
                fileFound = true;
                break;
            }
        }
        if (fileFound) {
            // MessageBox(wnd.getHWND(), file.c_str(), L"Drop", MB_OK);
            openFile(file);
        }
    }

   private:
    int oldX = 0;
    int oldY = 0;
    bool isDragging = false;
    Plotter plotter;
    Gdiplus::GraphicsPath* path;
};

WINMAIN(CheckerApp, L"Cut Checker", L"Cut Checker", windowWidth, windowHeight);