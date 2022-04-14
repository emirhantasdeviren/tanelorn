#include <string>
#include <windows.h>

#include "defines.hpp"
#include "window.hpp"

Window::Window(HINSTANCE instance, u32 width, u32 height, const std::string &title) {
    WNDCLASSEXA wcx = {
        sizeof(WNDCLASSEXA),     // cbSize
        CS_HREDRAW | CS_VREDRAW, // style
        DefWindowProcA,          // lpfnWndProc
        0,                       // cbClsExtra
        0,                       // cbWndExtra
        instance,                // hInstance
        nullptr,                 // hIcon
        nullptr,                 // hCursor
        nullptr,                 // hbrBackground
        nullptr,                 // lpszMenuName
        "TanelornWindowClass",   // lpszClassName
        nullptr                  // hIconSm
    };

    if (RegisterClassExA(&wcx) != 0) {
        this->handle =
            CreateWindowExA(0, wcx.lpszClassName, title.c_str(), WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                            10, 10, width, height, nullptr, nullptr, instance, nullptr);
    }
}

Window::~Window() {
    DestroyWindow(this->handle);
}

void Window::run() {
    for (;;) {
        MSG m;
        while (PeekMessageA(&m, this->handle, 0, 0, PM_REMOVE) > 0) {
            TranslateMessage(&m);
            DispatchMessageA(&m);
        }
    }
}