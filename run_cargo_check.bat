@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64
C:\Users\micha\.cargo\bin\cargo.exe check
