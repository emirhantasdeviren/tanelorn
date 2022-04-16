#pragma once

#include <windows.h>
#include <string>

#include "defines.hpp"

class Window {
  private:
    HWND handle;

  public:
    explicit Window(HINSTANCE instance, u32 width, u32 height, const std::string &title);
    ~Window();
    Window(const Window &w) = delete;

    void run();
};