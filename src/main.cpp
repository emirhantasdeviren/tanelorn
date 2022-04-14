#include <iostream>
#include <windows.h>
#include <stdlib.h>

#include "./window.hpp"
#include "./string.hpp"

// int WINAPI WinMain(HINSTANCE instance, HINSTANCE prev_instance, PSTR cmd_line, int cmd_show) {
int main() {
    tanelorn::String s = "Emirhan";

    char *p = s.as_c_str();
    for (i32 i = 0; i < 7; i++) {
        std::cout << *(p + i);
    }

    std::cout << std::endl;

    return 0;
}