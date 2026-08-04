//go:build windows

package main

import (
	"context"
	"embed"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"image"
	"image/draw"
	"image/png"
	"mime"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
	"time"
	"unsafe"
)

const (
	appName         = "月序"
	defaultWidth    = 3840
	defaultHeight   = 2160
	viewportBuffer  = 128
	wallpaperName   = "lunar-wallpaper.png"
	spiSetWallpaper = 0x0014
	spifUpdateIni   = 0x0001
	spifSendChange  = 0x0002
)

var version = "0.1.0-dev"

//go:embed index.html src/*
var webAssets embed.FS

type renderOptions struct {
	Width        int
	Height       int
	Year         int
	Theme        string
	Today        string
	Output       string
	SetWallpaper bool
	Quiet        bool
}

type settings struct {
	Theme string `json:"theme"`
}

func main() {
	enableDPIAwareness()

	update := flag.Bool("update", false, "生成并设置桌面壁纸")
	preview := flag.Bool("preview", false, "打开本地预览")
	quiet := flag.Bool("quiet", false, "不输出状态")
	showVersion := flag.Bool("version", false, "显示版本")
	width := flag.Int("width", defaultWidth, "壁纸宽度")
	height := flag.Int("height", defaultHeight, "壁纸高度")
	year := flag.Int("year", time.Now().Year(), "日历年份")
	theme := flag.String("theme", "", "主题：dark 或 light；指定后会记住")
	today := flag.String("today", time.Now().Format("2006-01-02"), "当天日期，格式 YYYY-MM-DD")
	output := flag.String("output", "", "生成图片位置")
	setWallpaper := flag.Bool("set-wallpaper", true, "将生成图设置为 Windows 壁纸")
	flag.Usage = usage
	flag.Parse()

	if *showVersion {
		fmt.Printf("%s %s\n", appName, version)
		return
	}
	if flag.NArg() == 1 && strings.HasPrefix(strings.ToLower(flag.Arg(0)), "yuexu://") {
		if err := handleProtocol(flag.Arg(0), *quiet); err != nil {
			exitWithError(err)
		}
		return
	}

	resolvedTheme, err := resolveTheme(*theme)
	if err != nil {
		exitWithError(err)
		return
	}
	if *preview {
		if err := openPreview(*year, resolvedTheme, *today); err != nil {
			exitWithError(err)
			return
		}
		return
	}

	// 双击运行和计划任务都走同一个一次性更新入口。
	if *update || flag.NArg() == 0 {
		err := renderWallpaper(renderOptions{
			Width:        *width,
			Height:       *height,
			Year:         *year,
			Theme:        resolvedTheme,
			Today:        *today,
			Output:       *output,
			SetWallpaper: *setWallpaper,
			Quiet:        *quiet,
		})
		if err != nil {
			exitWithError(err)
		}
		return
	}

	usage()
}

func resolveTheme(requested string) (string, error) {
	if requested != "" {
		if !validTheme(requested) {
			return "", errors.New("主题仅支持 dark 或 light")
		}
		if err := saveSettings(settings{Theme: requested}); err != nil {
			return "", err
		}
		return requested, nil
	}

	loaded, err := loadSettings()
	if err != nil {
		return "", err
	}
	if validTheme(loaded.Theme) {
		return loaded.Theme, nil
	}
	return "dark", nil
}

func validTheme(theme string) bool {
	return theme == "dark" || theme == "light"
}

func settingsFilePath() (string, error) {
	dir, err := applicationDataDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(dir, "settings.json"), nil
}

func loadSettings() (settings, error) {
	path, err := settingsFilePath()
	if err != nil {
		return settings{}, err
	}
	content, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return settings{}, nil
	}
	if err != nil {
		return settings{}, fmt.Errorf("读取设置：%w", err)
	}
	var loaded settings
	if err := json.Unmarshal(content, &loaded); err != nil {
		return settings{}, fmt.Errorf("解析设置：%w", err)
	}
	return loaded, nil
}

func saveSettings(next settings) error {
	path, err := settingsFilePath()
	if err != nil {
		return err
	}
	content, err := json.MarshalIndent(next, "", "  ")
	if err != nil {
		return err
	}
	temporary := path + ".tmp"
	if err := os.WriteFile(temporary, content, 0o600); err != nil {
		return fmt.Errorf("写入设置：%w", err)
	}
	if err := os.Rename(temporary, path); err != nil {
		return fmt.Errorf("保存设置：%w", err)
	}
	return nil
}

func handleProtocol(rawURL string, quiet bool) error {
	parsed, err := url.Parse(rawURL)
	if err != nil || parsed.Scheme != "yuexu" {
		return errors.New("无效的月序链接")
	}
	if parsed.Host != "theme" {
		return errors.New("不支持的月序操作")
	}
	theme := strings.Trim(strings.TrimPrefix(parsed.Path, "/"), " ")
	if !validTheme(theme) {
		return errors.New("主题仅支持 dark 或 light")
	}
	resolvedTheme, err := resolveTheme(theme)
	if err != nil {
		return err
	}
	return renderWallpaper(renderOptions{
		Width:        defaultWidth,
		Height:       defaultHeight,
		Year:         time.Now().Year(),
		Theme:        resolvedTheme,
		Today:        time.Now().Format("2006-01-02"),
		SetWallpaper: true,
		Quiet:        quiet,
	})
}

func enableDPIAwareness() {
	// 让无窗口 Chromium 继承真实显示比例，避免 Windows DPI 虚拟化裁掉截图视口。
	user32 := syscall.NewLazyDLL("user32.dll")
	_, _, _ = user32.NewProc("SetProcessDPIAware").Call()
}

func usage() {
	fmt.Fprintf(os.Stderr, "%s %s\n\n", appName, version)
	fmt.Fprintln(os.Stderr, "用法：")
	fmt.Fprintln(os.Stderr, "  LunarCalendar.exe --update --quiet")
	fmt.Fprintln(os.Stderr, "  LunarCalendar.exe --preview")
	fmt.Fprintln(os.Stderr, "  LunarCalendar.exe --update --theme light")
}

func reportError(err error) {
	fmt.Fprintln(os.Stderr, "月序更新失败：", err)
}

func exitWithError(err error) {
	reportError(err)
	os.Exit(1)
}

func renderWallpaper(options renderOptions) error {
	if options.Width < 800 || options.Height < 600 {
		return errors.New("壁纸尺寸至少需要 800×600")
	}
	if options.Year < 1900 || options.Year > 2100 {
		return errors.New("日历年份仅支持 1900-2100")
	}
	if !validTheme(options.Theme) {
		return errors.New("主题仅支持 dark 或 light")
	}
	if _, err := time.Parse("2006-01-02", options.Today); err != nil {
		return errors.New("当天日期格式应为 YYYY-MM-DD")
	}

	dataDir, err := applicationDataDir()
	if err != nil {
		return err
	}
	if options.Output == "" {
		options.Output = filepath.Join(dataDir, wallpaperName)
	}
	absoluteOutput, err := filepath.Abs(options.Output)
	if err != nil {
		return fmt.Errorf("解析输出路径：%w", err)
	}
	options.Output = absoluteOutput
	if err := os.MkdirAll(filepath.Dir(options.Output), 0o755); err != nil {
		return fmt.Errorf("创建输出目录：%w", err)
	}

	serverURL, closeServer, err := startAssetServer()
	if err != nil {
		return err
	}
	defer closeServer()

	renderProfile, err := os.MkdirTemp(dataDir, "render-")
	if err != nil {
		return fmt.Errorf("创建临时渲染目录：%w", err)
	}
	defer os.RemoveAll(renderProfile)

	captureOutput := options.Output + ".capture.png"
	if err := os.Remove(options.Output); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("清理旧壁纸：%w", err)
	}
	if err := os.Remove(captureOutput); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("清理旧截图：%w", err)
	}
	defer os.Remove(captureOutput)

	renderURL, err := buildRenderURL(serverURL, options)
	if err != nil {
		return err
	}
	browser, err := findBrowser()
	if err != nil {
		return err
	}

	args := []string{
		"--headless=new",
		"--disable-gpu",
		"--hide-scrollbars",
		"--disable-background-networking",
		"--disable-extensions",
		"--no-first-run",
		"--run-all-compositor-stages-before-draw",
		"--virtual-time-budget=1000",
		"--force-device-scale-factor=1",
		"--user-data-dir=" + renderProfile,
		fmt.Sprintf("--window-size=%d,%d", options.Width, options.Height+viewportBuffer),
		"--screenshot=" + captureOutput,
		renderURL,
	}
	command := exec.Command(browser, args...)
	command.SysProcAttr = &syscall.SysProcAttr{HideWindow: true}
	if output, err := command.CombinedOutput(); err != nil {
		return fmt.Errorf("浏览器渲染失败：%w%s", err, compactProcessOutput(output))
	}

	if err := cropPNG(captureOutput, options.Output, options.Width, options.Height); err != nil {
		return err
	}

	info, err := os.Stat(options.Output)
	if err != nil || info.Size() == 0 {
		return errors.New("未生成有效壁纸图片")
	}
	if options.SetWallpaper {
		if err := setDesktopWallpaper(options.Output); err != nil {
			return err
		}
	}
	if !options.Quiet {
		fmt.Printf("已生成：%s\n", options.Output)
		if options.SetWallpaper {
			fmt.Println("已设置为 Windows 桌面背景。")
		}
	}
	return nil
}

func cropPNG(sourcePath, targetPath string, width, height int) error {
	sourceFile, err := os.Open(sourcePath)
	if err != nil {
		return fmt.Errorf("读取浏览器截图：%w", err)
	}
	defer sourceFile.Close()

	source, err := png.Decode(sourceFile)
	if err != nil {
		return fmt.Errorf("解析浏览器截图：%w", err)
	}
	bounds := source.Bounds()
	if bounds.Dx() < width || bounds.Dy() < height {
		return fmt.Errorf("浏览器截图尺寸不足：%d×%d", bounds.Dx(), bounds.Dy())
	}

	target := image.NewRGBA(image.Rect(0, 0, width, height))
	draw.Draw(target, target.Bounds(), source, bounds.Min, draw.Src)
	targetFile, err := os.Create(targetPath)
	if err != nil {
		return fmt.Errorf("写入壁纸：%w", err)
	}
	defer targetFile.Close()
	if err := png.Encode(targetFile, target); err != nil {
		return fmt.Errorf("编码壁纸：%w", err)
	}
	return nil
}

func compactProcessOutput(output []byte) string {
	text := strings.TrimSpace(string(output))
	if text == "" {
		return ""
	}
	if len(text) > 480 {
		text = text[len(text)-480:]
	}
	return "\n" + text
}

func buildRenderURL(serverURL string, options renderOptions) (string, error) {
	parsed, err := url.Parse(serverURL + "/index.html")
	if err != nil {
		return "", err
	}
	query := parsed.Query()
	query.Set("wallpaper", "1")
	query.Set("width", fmt.Sprint(options.Width))
	query.Set("height", fmt.Sprint(options.Height))
	query.Set("year", fmt.Sprint(options.Year))
	query.Set("theme", options.Theme)
	query.Set("today", options.Today)
	query.Set("render", fmt.Sprint(time.Now().UnixNano()))
	parsed.RawQuery = query.Encode()
	return parsed.String(), nil
}

func applicationDataDir() (string, error) {
	base := os.Getenv("LOCALAPPDATA")
	if base == "" {
		return "", errors.New("未找到 LOCALAPPDATA 目录")
	}
	dir := filepath.Join(base, "YueXu")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return "", fmt.Errorf("创建应用目录：%w", err)
	}
	return dir, nil
}

func startAssetServer() (string, func(), error) {
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return "", nil, fmt.Errorf("启动本地渲染服务：%w", err)
	}
	mux := http.NewServeMux()
	mux.HandleFunc("/", serveEmbeddedAsset)
	server := &http.Server{Handler: mux, ReadHeaderTimeout: 5 * time.Second}
	go func() {
		_ = server.Serve(listener)
	}()
	closeServer := func() {
		ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		_ = server.Shutdown(ctx)
	}
	return "http://" + listener.Addr().String(), closeServer, nil
}

func serveEmbeddedAsset(writer http.ResponseWriter, request *http.Request) {
	path := strings.TrimPrefix(request.URL.Path, "/")
	if path == "" {
		path = "index.html"
	}
	if strings.Contains(path, "..") {
		http.NotFound(writer, request)
		return
	}
	content, err := webAssets.ReadFile(path)
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	if contentType := mime.TypeByExtension(filepath.Ext(path)); contentType != "" {
		writer.Header().Set("Content-Type", contentType)
	}
	writer.Header().Set("Cache-Control", "no-store")
	_, _ = writer.Write(content)
}

func findBrowser() (string, error) {
	programFiles := []string{os.Getenv("ProgramFiles(x86)"), os.Getenv("ProgramFiles")}
	candidates := make([]string, 0, 6)
	for _, root := range programFiles {
		if root == "" {
			continue
		}
		candidates = append(candidates,
			filepath.Join(root, "Google", "Chrome", "Application", "chrome.exe"),
			filepath.Join(root, "Microsoft", "Edge", "Application", "msedge.exe"),
		)
	}
	for _, candidate := range candidates {
		if info, err := os.Stat(candidate); err == nil && !info.IsDir() {
			return candidate, nil
		}
	}
	return "", errors.New("未找到 Microsoft Edge 或 Google Chrome")
}

func openPreview(year int, theme, today string) error {
	if year < 1900 || year > 2100 {
		return errors.New("日历年份仅支持 1900-2100")
	}
	if !validTheme(theme) {
		return errors.New("主题仅支持 dark 或 light")
	}
	dataDir, err := applicationDataDir()
	if err != nil {
		return err
	}
	previewDir := filepath.Join(dataDir, "preview")
	for _, asset := range []string{"index.html", "src/styles.css", "src/lunar.js", "src/calendar.js", "src/app.js"} {
		content, err := webAssets.ReadFile(asset)
		if err != nil {
			return fmt.Errorf("读取内嵌资源 %s：%w", asset, err)
		}
		target := filepath.Join(previewDir, filepath.FromSlash(asset))
		if err := os.MkdirAll(filepath.Dir(target), 0o755); err != nil {
			return err
		}
		if err := os.WriteFile(target, content, 0o644); err != nil {
			return err
		}
	}
	browser, err := findBrowser()
	if err != nil {
		return err
	}
	previewURL := "file:///" + filepath.ToSlash(filepath.Join(previewDir, "index.html"))
	query := url.Values{"year": {fmt.Sprint(year)}, "theme": {theme}, "today": {today}, "native": {"1"}}
	command := exec.Command(browser, previewURL+"?"+query.Encode())
	command.SysProcAttr = &syscall.SysProcAttr{HideWindow: false}
	return command.Start()
}

func setDesktopWallpaper(path string) error {
	wallpaperPath, err := syscall.UTF16PtrFromString(path)
	if err != nil {
		return fmt.Errorf("壁纸路径无效：%w", err)
	}
	user32 := syscall.NewLazyDLL("user32.dll")
	procedure := user32.NewProc("SystemParametersInfoW")
	result, _, callErr := procedure.Call(
		uintptr(spiSetWallpaper),
		0,
		uintptr(unsafe.Pointer(wallpaperPath)),
		uintptr(spifUpdateIni|spifSendChange),
	)
	if result == 0 {
		return fmt.Errorf("Windows 拒绝设置壁纸：%w", callErr)
	}
	return nil
}
