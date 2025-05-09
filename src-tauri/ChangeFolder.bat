@echo off
REM 切換到此批次檔所在的資料夾
cd /d %~dp0

echo 正在清理專案...
cargo clean

echo 正在建置專案...
cargo build

echo.
echo 建置完成，按任意鍵繼續...
pause
