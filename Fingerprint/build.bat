@echo off
g++ -g helloworld.cpp -o helloworld.exe -I"./lib" -L"C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.22000.0\\um\\x64" -lWinBio -lkernel32
if %ERRORLEVEL% == 0 (
    echo Build succeeded.
) else (
    echo Build failed.
)
pause