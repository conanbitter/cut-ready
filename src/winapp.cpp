#include "winapp.hpp"
#include <windowsx.h>

void Window::createBackbufferImage(HDC mainHdc, int width, int height) {
    if (backbufferBitmap) {
        DeleteObject(backbufferBitmap);
    }
    backbufferBitmap = CreateCompatibleBitmap(mainHdc, width, height);
    backbufferRect.right = width;
    backbufferRect.bottom = height;
    area.Width = width;
    area.Height = height;
    SelectObject(backbufferHdc, backbufferBitmap);
}

void Window::initBackbuffer(HDC mainHdc, int width, int height) {
    backbufferRect.left = 0;
    backbufferRect.top = 0;
    area.X = 0;
    area.Y = 0;
    backbufferHdc = CreateCompatibleDC(mainHdc);
    createBackbufferImage(mainHdc, width, height);
}

void Window::freeBackbuffer() {
    DeleteObject(backbufferBitmap);
    DeleteDC(backbufferHdc);
}

Window::Window(HINSTANCE hInstance, const wchar_t *classname, const wchar_t *title, int width, int height) {
    Gdiplus::GdiplusStartupInput gdiplusStartupInput;

    WNDCLASSEX wc = {};
    Gdiplus::GdiplusStartup(&gdiplusToken, &gdiplusStartupInput, NULL);

    wc.cbSize = sizeof(WNDCLASSEX);
    wc.hbrBackground = (HBRUSH)GetStockObject(WHITE_BRUSH);
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = classname;
    wc.hCursor = LoadCursor(0, IDC_ARROW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.cbWndExtra = 0;
    wc.cbClsExtra = 0;
    wc.lpszMenuName = NULL;

    RegisterClassEx(&wc);

    // Calculate window geometry

    SIZE screenSize;
    LONG winX, winY;
    screenSize.cx = GetSystemMetrics(SM_CXSCREEN);
    screenSize.cy = GetSystemMetrics(SM_CYSCREEN);

    winX = (screenSize.cx - width) / 2;
    winY = (screenSize.cy - height) / 2;

    RECT winRect = {winX, winY, winX + width, winY + height};

    AdjustWindowRectEx(&winRect, WS_OVERLAPPEDWINDOW, FALSE, WS_EX_OVERLAPPEDWINDOW);

    // Create the window.

    hwnd = CreateWindowEx(
        WS_EX_OVERLAPPEDWINDOW,
        classname,
        title,
        WS_OVERLAPPEDWINDOW,

        // Size and position
        winRect.left, winRect.top,
        winRect.right - winRect.left, winRect.bottom - winRect.top,

        NULL,
        NULL,
        hInstance,
        NULL);

    if (hwnd == NULL) {
        /* ERROR */
    }

    SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)this);

    ShowWindow(hwnd, SW_NORMAL);
    HDC wdc = GetDC(hwnd);
    initBackbuffer(wdc, width, height);
}

void Window::setApp(Application &app) {
    this->app = &app;
}

void Window::run() {
    app->onInit();
    MSG msg = {};
    while (GetMessage(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

void Window::exit(HINSTANCE hInstance, const wchar_t *classname) {
    app->onExit();
    freeBackbuffer();
    DestroyWindow(hwnd);
    UnregisterClass(classname, hInstance);
    Gdiplus::GdiplusShutdown(gdiplusToken);
}

LRESULT CALLBACK Window::WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    Window *wnd = (Window *)GetWindowLongPtr(hwnd, GWLP_USERDATA);
    switch (uMsg) {
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;

        case WM_LBUTTONDOWN:
            wnd->app->onMouseDown(BUTTON_LEFT, GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam));
            return 0;

        case WM_LBUTTONUP:
            wnd->app->onMouseUp(BUTTON_LEFT, GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam));
            return 0;

        case WM_MOUSEWHEEL: {
            int delta = GET_WHEEL_DELTA_WPARAM(wParam) / WHEEL_DELTA;
            int direction;
            if (delta >= 0) {
                direction = 1;
            } else {
                delta = -delta;
                direction = -1;
            }
            POINT coord;
            coord.x = GET_X_LPARAM(lParam);
            coord.y = GET_Y_LPARAM(lParam);
            ScreenToClient(hwnd, &coord);
            for (int i = 0; i < delta; i++) {
                wnd->app->onMouseWheel(direction, coord.x, coord.y);
            }
        }
            return 0;

        case WM_MOUSEMOVE: {
            int buttons = 0;
            if (wParam & MK_LBUTTON) buttons |= BUTTON_LEFT;
            if (wParam & MK_RBUTTON) buttons |= BUTTON_RIGHT;
            if (wParam & MK_MBUTTON) buttons |= BUTTON_MIDDLE;
            wnd->app->onMouseMove(buttons, GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam));
        }
            return 0;

        case WM_ERASEBKGND:
            return TRUE;

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            Gdiplus::Graphics graphics(wnd->backbufferHdc);
            wnd->app->onDraw(graphics, wnd->area);
            BitBlt(hdc,
                   ps.rcPaint.left,
                   ps.rcPaint.top,
                   ps.rcPaint.right - ps.rcPaint.left,
                   ps.rcPaint.bottom - ps.rcPaint.top,
                   wnd->backbufferHdc,
                   ps.rcPaint.left,
                   ps.rcPaint.top,
                   SRCCOPY);
            EndPaint(hwnd, &ps);
        }
            return 0;

        case WM_SIZE: {
            UINT width = LOWORD(lParam);
            UINT height = HIWORD(lParam);
            HDC wdc = GetDC(hwnd);
            wnd->createBackbufferImage(wdc, width, height);
            if (wnd->app) wnd->app->onResize(width, height);
        }
            return 0;

        case WM_DROPFILES: {
            unsigned int fileCount = DragQueryFile((HDROP)wParam, 0xFFFFFFFF, nullptr, 0);
            std::vector<std::wstring> files;
            for (int i = 0; i < fileCount; i++) {
                size_t pathLength = DragQueryFile((HDROP)wParam, i, nullptr, 0) + 1;
                wchar_t *path_buffer = (wchar_t *)malloc(pathLength * sizeof(wchar_t));
                DragQueryFile((HDROP)wParam, i, path_buffer, pathLength);
                files.push_back(std::wstring(path_buffer));
                delete (path_buffer);
            }
            wnd->app->onFilesDrop(files);
        }
            return 0;
    }
    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

void Window::update() {
    InvalidateRect(hwnd, NULL, FALSE);
}

HWND Window::getHWND() {
    return hwnd;
}

void Window::setTitle(const wchar_t *title) {
    SetWindowText(hwnd, title);
}

void Window::setMouseCapture(bool capture) {
    if (capture) {
        SetCapture(hwnd);
    } else {
        ReleaseCapture();
    }
}

void Window::setAcceptDropFiles(bool accept) {
    DragAcceptFiles(hwnd, accept);
}