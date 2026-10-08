#pragma once

#ifndef UNICODE
#define UNICODE
#endif

#include <windows.h>
#include <gdiplus.h>
#include <string>
#include <vector>

const int BUTTON_LEFT = 0b001;
const int BUTTON_RIGHT = 0b010;
const int BUTTON_MIDDLE = 0b100;

class Window;

class Application {
   public:
    Application(Window &parent, std::wstring arg) : wnd{parent}, argument{arg} {}
    virtual void onInit() {}
    virtual void onDraw(Gdiplus::Graphics &gfx, Gdiplus::Rect &area) {}
    virtual void onExit() {}
    virtual void onMouseDown(int button, int x, int y) {}
    virtual void onMouseMove(int buttons, int x, int y) {}
    virtual void onMouseUp(int button, int x, int y) {}
    virtual void onMouseWheel(int direction, int x, int y) {}
    virtual void onResize(int newWidth, int newHeight) {}
    virtual void onFilesDrop(std::vector<std::wstring> filelist) {}

   protected:
    Window &wnd;
    std::wstring argument;

    friend Window;
};

class Window {
   public:
    Window(HINSTANCE hInstance, const wchar_t *classname, const wchar_t *title, int width, int height);
    void exit(HINSTANCE hInstance, const wchar_t *classname);
    void setApp(Application &app);
    void run();
    void update();
    void setTitle(const wchar_t *title);
    void setMouseCapture(bool capture);
    void setAcceptDropFiles(bool accept);
    HWND getHWND();
    static LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);

   private:
    HWND hwnd;
    ULONG_PTR gdiplusToken;
    Application *app = nullptr;

    HDC backbufferHdc;
    HBITMAP backbufferBitmap;
    RECT backbufferRect;
    Gdiplus::Rect area;

    void createBackbufferImage(HDC mainHdc, int width, int height);
    void initBackbuffer(HDC mainHdc, int width, int height);
    void freeBackbuffer();
};

#define WINMAIN(appclass, title, classname, width, height)                                            \
    int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, PWSTR pCmdLine, int nCmdShow) { \
        const wchar_t CLASS_NAME[] = classname;                                                       \
        Window window(hInstance, CLASS_NAME, title, width, height);                                   \
        appclass app(window, pCmdLine);                                                               \
        window.setApp(app);                                                                           \
        window.run();                                                                                 \
        window.exit(hInstance, CLASS_NAME);                                                           \
        return 0;                                                                                     \
    }