@echo off
echo Checking Rust toolchain and target...

:: Check if rustup is installed
where rustup >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: rustup is not installed or not in PATH.
    exit /b 1
)

:: Check if stable-x86_64-pc-windows-gnu is installed
rustup toolchain list | findstr "stable-x86_64-pc-windows-gnu" >nul
if %ERRORLEVEL% neq 0 (
    echo stable-x86_64-pc-windows-gnu toolchain not found. Installing...
    rustup toolchain install stable-x86_64-pc-windows-gnu
    if %ERRORLEVEL% neq 0 (
        echo Failed to install GNU toolchain.
        exit /b 1
    )
) else (
    echo GNU toolchain is already installed.
)

:: Check if it's set as default
rustup default | findstr "stable-x86_64-pc-windows-gnu" >nul
if %ERRORLEVEL% neq 0 (
    echo Setting stable-x86_64-pc-windows-gnu as default...
    rustup default stable-x86_64-pc-windows-gnu
) else (
    echo GNU toolchain is already the default.
)

:: Check if clang or gcc is available in PATH
where clang >nul 2>nul
if %ERRORLEVEL% neq 0 (
    where gcc >nul 2>nul
    if %ERRORLEVEL% neq 0 (
        echo Warning: Neither clang nor gcc was found in PATH.
        echo Please ensure LLVM-MinGW or MSYS2 is installed and added to PATH.
    ) else (
        echo Found gcc in PATH.
    )
) else (
    echo Found clang in PATH.
)

echo.
echo Running cargo check...
cargo check
