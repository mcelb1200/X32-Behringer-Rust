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

:: Ensure .vscode configurations exist for new users
if not exist .vscode (
    mkdir .vscode
)
if not exist .vscode\settings.json (
    echo Creating .vscode\settings.json for IDE integration...
    (
        echo {
        echo   "rust-analyzer.cargo.target": "x86_64-pc-windows-gnu",
        echo   "rust-analyzer.check.command": "clippy",
        echo   "rust-analyzer.check.allTargets": false,
        echo   "rust-analyzer.cargo.buildScripts.enable": true,
        echo   "rust-analyzer.procMacro.enable": true,
        echo   "[rust]": {
        echo     "editor.defaultFormatter": "rust-lang.rust-analyzer",
        echo     "editor.formatOnSave": true
        echo   },
        echo   "// rust-analyzer.testExplorer.runType": "cargo-nextest",
        echo   "// rust-analyzer.testExplorer.coverageType": "cargo-llvm-cov"
        echo }
    ) > .vscode\settings.json
)
if not exist .vscode\tasks.json (
    echo Creating .vscode\tasks.json for tasks integration...
    (
        echo {
        echo   "version": "2.0.0",
        echo   "tasks": [
        echo     {
        echo       "label": "Cargo Check Workspace",
        echo       "type": "shell",
        echo       "command": "cargo check --workspace",
        echo       "problemMatcher": ["$rustc-json"],
        echo       "group": "build"
        echo     },
        echo     {
        echo       "label": "Cargo Clippy Workspace",
        echo       "type": "shell",
        echo       "command": "cargo clippy --workspace",
        echo       "problemMatcher": ["$rustc-json"],
        echo       "group": "build"
        echo     },
        echo     {
        echo       "label": "Run Tests",
        echo       "type": "shell",
        echo       "command": "powershell",
        echo       "args": [
        echo         "-ExecutionPolicy",
        echo         "Bypass",
        echo         "-File",
        echo         "${workspaceFolder}/run_tests.ps1",
        echo         "-Mode",
        echo         "non_interactive"
        echo       ],
        echo       "group": {
        echo         "kind": "test",
        echo         "isDefault": true
        echo       }
        echo     }
        echo   ]
        echo }
    ) > .vscode\tasks.json
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
