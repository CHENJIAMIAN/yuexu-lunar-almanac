package main

import (
	"image"
	"image/color"
	"image/png"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"testing"
)

func TestBuildRenderURL(t *testing.T) {
	value, err := buildRenderURL("http://127.0.0.1:12345", renderOptions{
		Width: 3840, Height: 2160, Year: 2026, Theme: "dark", Today: "2026-08-04",
	})
	if err != nil {
		t.Fatalf("buildRenderURL returned error: %v", err)
	}
	parsed, err := url.Parse(value)
	if err != nil {
		t.Fatalf("parse URL: %v", err)
	}
	if parsed.Path != "/index.html" {
		t.Fatalf("unexpected path: %s", parsed.Path)
	}
	query := parsed.Query()
	if query.Get("wallpaper") != "1" || query.Get("width") != "3840" || query.Get("height") != "2160" {
		t.Fatalf("missing render parameters: %s", parsed.RawQuery)
	}
	if query.Get("theme") != "dark" || query.Get("today") != "2026-08-04" {
		t.Fatalf("unexpected theme/date query: %s", parsed.RawQuery)
	}
}

func TestCropPNG(t *testing.T) {
	tempDir := t.TempDir()
	sourcePath := filepath.Join(tempDir, "source.png")
	targetPath := filepath.Join(tempDir, "target.png")
	source := image.NewRGBA(image.Rect(0, 0, 4, 3))
	for y := 0; y < 3; y++ {
		for x := 0; x < 4; x++ {
			source.SetRGBA(x, y, color.RGBA{R: uint8(x * 40), G: uint8(y * 50), B: 25, A: 255})
		}
	}
	file, err := os.Create(sourcePath)
	if err != nil {
		t.Fatal(err)
	}
	if err := png.Encode(file, source); err != nil {
		file.Close()
		t.Fatal(err)
	}
	file.Close()

	if err := cropPNG(sourcePath, targetPath, 2, 2); err != nil {
		t.Fatalf("cropPNG returned error: %v", err)
	}
	targetFile, err := os.Open(targetPath)
	if err != nil {
		t.Fatal(err)
	}
	target, err := png.Decode(targetFile)
	targetFile.Close()
	if err != nil {
		t.Fatal(err)
	}
	if target.Bounds().Dx() != 2 || target.Bounds().Dy() != 2 {
		t.Fatalf("unexpected cropped size: %v", target.Bounds())
	}
	if got := color.RGBAModel.Convert(target.At(1, 1)).(color.RGBA); got.R != 40 || got.G != 50 || got.B != 25 {
		t.Fatalf("unexpected cropped pixel: %#v", got)
	}
}

func TestServeEmbeddedAsset(t *testing.T) {
	request := httptest.NewRequest("GET", "http://localhost/index.html", nil)
	response := httptest.NewRecorder()
	serveEmbeddedAsset(response, request)
	if response.Code != 200 {
		t.Fatalf("unexpected response code: %d", response.Code)
	}
	if len(response.Body.Bytes()) == 0 {
		t.Fatal("embedded index.html is empty")
	}

	missing := httptest.NewRecorder()
	serveEmbeddedAsset(missing, httptest.NewRequest("GET", "http://localhost/../secret", nil))
	if missing.Code != 404 {
		t.Fatalf("path traversal should be rejected, got %d", missing.Code)
	}
}

func TestValidTheme(t *testing.T) {
	if !validTheme("dark") || !validTheme("light") {
		t.Fatal("supported themes must be accepted")
	}
	if validTheme("forest") || validTheme("") {
		t.Fatal("unsupported themes must be rejected")
	}
}
