@echo off

pushd target\debug
cl /Zi /EHsc /std:c++20 ..\..\src\main.cpp ..\..\src\window.cpp ..\..\src\string.cpp user32.lib
popd