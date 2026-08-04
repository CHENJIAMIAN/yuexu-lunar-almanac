package main

import (
	"bytes"
	"encoding/binary"
	"flag"
	"image"
	"image/color"
	"image/png"
	"os"
)

func main() {
	output := flag.String("output", "assets/YueXu.ico", "ICO 输出路径")
	flag.Parse()

	canvas := image.NewNRGBA(image.Rect(0, 0, 256, 256))
	fill(canvas, color.NRGBA{R: 25, G: 33, B: 28, A: 255})
	roundedSquare(canvas, 18, 18, 220, 220, 48, color.NRGBA{R: 32, G: 43, B: 36, A: 255})
	circle(canvas, 128, 126, 74, color.NRGBA{R: 241, G: 234, B: 223, A: 255})
	circle(canvas, 157, 100, 74, color.NRGBA{R: 32, G: 43, B: 36, A: 255})
	circle(canvas, 176, 170, 17, color.NRGBA{R: 223, G: 107, B: 84, A: 255})

	var pngData bytes.Buffer
	if err := png.Encode(&pngData, canvas); err != nil {
		panic(err)
	}
	file, err := os.Create(*output)
	if err != nil {
		panic(err)
	}
	defer file.Close()

	// Windows ICO can directly carry a PNG image at 256×256.
	_ = binary.Write(file, binary.LittleEndian, uint16(0))
	_ = binary.Write(file, binary.LittleEndian, uint16(1))
	_ = binary.Write(file, binary.LittleEndian, uint16(1))
	_, _ = file.Write([]byte{0, 0, 0, 0})
	_ = binary.Write(file, binary.LittleEndian, uint16(1))
	_ = binary.Write(file, binary.LittleEndian, uint16(32))
	_ = binary.Write(file, binary.LittleEndian, uint32(pngData.Len()))
	_ = binary.Write(file, binary.LittleEndian, uint32(22))
	_, _ = file.Write(pngData.Bytes())
}

func fill(canvas *image.NRGBA, value color.NRGBA) {
	for y := 0; y < canvas.Bounds().Dy(); y++ {
		for x := 0; x < canvas.Bounds().Dx(); x++ {
			canvas.SetNRGBA(x, y, value)
		}
	}
}

func roundedSquare(canvas *image.NRGBA, x, y, width, height, radius int, value color.NRGBA) {
	for py := y; py < y+height; py++ {
		for px := x; px < x+width; px++ {
			dx := 0
			dy := 0
			if px < x+radius { dx = x + radius - px }
			if px > x+width-radius { dx = px - (x + width - radius) }
			if py < y+radius { dy = y + radius - py }
			if py > y+height-radius { dy = py - (y + height - radius) }
			if dx*dx+dy*dy <= radius*radius { canvas.SetNRGBA(px, py, value) }
		}
	}
}

func circle(canvas *image.NRGBA, cx, cy, radius int, value color.NRGBA) {
	for y := cy - radius; y <= cy+radius; y++ {
		for x := cx - radius; x <= cx+radius; x++ {
			dx, dy := x-cx, y-cy
			if dx*dx+dy*dy <= radius*radius && image.Pt(x, y).In(canvas.Bounds()) { canvas.SetNRGBA(x, y, value) }
		}
	}
}
