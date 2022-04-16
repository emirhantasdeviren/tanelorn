#include <iostream>
#include <windows.h>
#include <stdlib.h>

#include "./window.hpp"
#include "./string.hpp"

int WINAPI WinMain(HINSTANCE instance, HINSTANCE prev_instance, PSTR cmd_line, int cmd_show) {
    Window w(instance, 1280, 720, "Tanelorn Engine");

    w.run();

    return 0;
}