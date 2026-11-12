// Command packager erzeugt die auslieferbaren Artefakte:
//
//	app   - macOS .app-Bundle (Info.plist, Icon, Binary)
//	icns  - macOS Icon (.icns) aus einem PNG
//	ico   - Windows Icon (.ico) aus einem PNG
//	usb   - USB-Stick-Aufbau (Dateien kopieren, README schreiben)
package main

import (
	"encoding/binary"
	"fmt"
	"image"
	"image/color"
	"image/draw"
	"image/png"
	"os"
	"path/filepath"
	"strings"
)

const (
	bundleID      = "com.legl3r.screenrecv1"
	appName       = "macREC - Aufnahmesoftware"
	bundleVersion = "1.0.0"
)

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(2)
	}
	var err error
	switch os.Args[1] {
	case "app":
		err = cmdApp(os.Args[2:])
	case "icns":
		err = cmdIcns(os.Args[2:])
	case "ico":
		err = cmdIco(os.Args[2:])
	case "usb":
		err = cmdUsb(os.Args[2:])
	case "png":
		err = cmdPng(os.Args[2:])
	default:
		usage()
		os.Exit(2)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "Fehler:", err)
		os.Exit(1)
	}
}

func usage() {
	fmt.Println(`packager - erzeugt .app-Bundle, Icons und den USB-Stick-Aufbau

  packager app  --bin <binary> --icon <png> --out <macREC.app>
  packager icns --png <png> --out <icon.icns>
  packager ico  --png <png> --out <icon.ico>
  packager usb  --root <ziel> [--mac <app>] [--win <exe>]
  packager png  --in <png> --out <png> [--size <pixel>]`)
}

func flagValue(args []string, name string) (string, bool) {
	for i, arg := range args {
		if arg == "--"+name && i+1 < len(args) {
			return args[i+1], true
		}
	}
	return "", false
}

// ---------------------------------------------------------------- app bundle

func cmdApp(args []string) error {
	bin, ok := flagValue(args, "bin")
	if !ok {
		return fmt.Errorf("--bin fehlt")
	}
	out, ok := flagValue(args, "out")
	if !ok {
		return fmt.Errorf("--out fehlt")
	}
	icon, _ := flagValue(args, "icon")

	contents := filepath.Join(out, "Contents")
	macos := filepath.Join(contents, "MacOS")
	resources := filepath.Join(contents, "Resources")
	for _, dir := range []string{macos, resources} {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return err
		}
	}

	if err := copyFile(bin, filepath.Join(macos, "screenrecv"), 0o755); err != nil {
		return fmt.Errorf("Binary kopieren: %w", err)
	}

	if icon != "" {
		icnsPath := filepath.Join(resources, "AppIcon.icns")
		if err := writeIcns(icon, icnsPath, []int{16, 32, 128, 256, 512}); err != nil {
			return fmt.Errorf("Icon: %w", err)
		}
	}

	plist := fmt.Sprintf(`<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>%s</string>
    <key>CFBundleDisplayName</key><string>%s</string>
    <key>CFBundleIdentifier</key><string>%s</string>
    <key>CFBundleExecutable</key><string>screenrecv</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundleShortVersionString</key><string>%s</string>
    <key>CFBundleVersion</key><string>%s</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
    <key>NSHumanReadableCopyright</key><string>(C) 2026 Niklas Legler - niklaslegler.at</string>
</dict>
</plist>
`, appName, appName, bundleID, bundleVersion, bundleVersion)

	if err := os.WriteFile(filepath.Join(contents, "Info.plist"), []byte(plist), 0o644); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(contents, "PkgInfo"), []byte("APPL????"), 0o644); err != nil {
		return err
	}

	fmt.Println("App-Bundle erstellt:", out)
	return nil
}

// cmdPng normalisiert ein PNG (8 Bit, RGBA) und skaliert es optional.
func cmdPng(args []string) error {
	source, ok := flagValue(args, "in")
	if !ok {
		return fmt.Errorf("--in fehlt")
	}
	target, ok := flagValue(args, "out")
	if !ok {
		return fmt.Errorf("--out fehlt")
	}
	img, err := loadPNG(source)
	if err != nil {
		return err
	}
	if sizeText, ok := flagValue(args, "size"); ok {
		var size int
		if _, err := fmt.Sscanf(sizeText, "%d", &size); err != nil || size <= 0 {
			return fmt.Errorf("--size ungueltig: %s", sizeText)
		}
		img = resize(img, size)
	}
	data, err := encodePNG(img)
	if err != nil {
		return err
	}
	if err := os.WriteFile(target, data, 0o644); err != nil {
		return err
	}
	fmt.Println("png:", target)
	return nil
}

func cmdIcns(args []string) error {
	source, ok := flagValue(args, "png")
	if !ok {
		return fmt.Errorf("--png fehlt")
	}
	target, ok := flagValue(args, "out")
	if !ok {
		return fmt.Errorf("--out fehlt")
	}
	if err := writeIcns(source, target, []int{16, 32, 128, 256, 512}); err != nil {
		return err
	}
	fmt.Println("icns:", target)
	return nil
}

func cmdIco(args []string) error {
	source, ok := flagValue(args, "png")
	if !ok {
		return fmt.Errorf("--png fehlt")
	}
	target, ok := flagValue(args, "out")
	if !ok {
		return fmt.Errorf("--out fehlt")
	}
	if err := writeIco(source, target, []int{16, 32, 48, 64, 128, 256}); err != nil {
		return err
	}
	fmt.Println("ico:", target)
	return nil
}

// -------------------------------------------------------------------- icons

// resize skaliert ein Bild mit einem einfachen Box-Filter.
func resize(src image.Image, size int) *image.NRGBA {
	dst := image.NewNRGBA(image.Rect(0, 0, size, size))
	draw.Draw(dst, dst.Bounds(), image.NewUniform(color.NRGBA{}), image.Point{}, draw.Src)
	bounds := src.Bounds()
	for y := 0; y < size; y++ {
		for x := 0; x < size; x++ {
			r, g, b, a := src.At(
				bounds.Min.X+x*bounds.Dx()/size,
				bounds.Min.Y+y*bounds.Dy()/size,
			).RGBA()
			dst.Set(x, y, color.NRGBA{
				R: uint8(r >> 8),
				G: uint8(g >> 8),
				B: uint8(b >> 8),
				A: uint8(a >> 8),
			})
		}
	}
	return dst
}

func loadPNG(path string) (image.Image, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	return png.Decode(file)
}

func encodePNG(img image.Image) ([]byte, error) {
	var buf strings.Builder
	encoder := png.Encoder{}
	if err := encoder.Encode(&buf, img); err != nil {
		return nil, err
	}
	return []byte(buf.String()), nil
}

// writeIcns schreibt ein Apple-Icon mit PNG-Inhalten.
func writeIcns(source string, target string, sizes []int) error {
	img, err := loadPNG(source)
	if err != nil {
		return err
	}

	type entry struct {
		kind string
		size int
	}
	kinds := map[int]string{
		16:  "icp4",
		32:  "icp5",
		128: "ic07",
		256: "ic08",
		512: "ic09",
	}

	var payload []byte
	for _, size := range sizes {
		kind, ok := kinds[size]
		if !ok {
			continue
		}
		data, err := encodePNG(resize(img, size))
		if err != nil {
			return err
		}
		header := make([]byte, 8)
		copy(header[0:4], kind)
		binary.BigEndian.PutUint32(header[4:8], uint32(len(data)+8))
		payload = append(payload, header...)
		payload = append(payload, data...)
	}

	out := make([]byte, 0, len(payload)+8)
	out = append(out, []byte("icns")...)
	length := make([]byte, 4)
	binary.BigEndian.PutUint32(length, uint32(len(payload)+8))
	out = append(out, length...)
	out = append(out, payload...)

	return os.WriteFile(target, out, 0o644)
}

// writeIco schreibt ein Windows-Icon mit PNG-Inhalten.
func writeIco(source string, target string, sizes []int) error {
	img, err := loadPNG(source)
	if err != nil {
		return err
	}

	type entry struct {
		size int
		data []byte
	}
	var entries []entry
	for _, size := range sizes {
		data, err := encodePNG(resize(img, size))
		if err != nil {
			return err
		}
		entries = append(entries, entry{size: size, data: data})
	}

	header := make([]byte, 6)
	binary.LittleEndian.PutUint16(header[0:2], 0) // reserviert
	binary.LittleEndian.PutUint16(header[2:4], 1) // Typ: Icon
	binary.LittleEndian.PutUint16(header[4:6], uint16(len(entries)))

	offset := 6 + 16*len(entries)
	var directory []byte
	var images []byte
	for _, item := range entries {
		dimension := byte(item.size)
		if item.size >= 256 {
			dimension = 0
		}
		record := make([]byte, 16)
		record[0] = dimension
		record[1] = dimension
		record[2] = 0                                  // Farbpalette
		record[3] = 0                                  // reserviert
		binary.LittleEndian.PutUint16(record[4:6], 1)  // Farbebenen
		binary.LittleEndian.PutUint16(record[6:8], 32) // Bits pro Pixel
		binary.LittleEndian.PutUint32(record[8:12], uint32(len(item.data)))
		binary.LittleEndian.PutUint32(record[12:16], uint32(offset))
		directory = append(directory, record...)
		images = append(images, item.data...)
		offset += len(item.data)
	}

	out := append(header, directory...)
	out = append(out, images...)
	return os.WriteFile(target, out, 0o644)
}

// ----------------------------------------------------------------- usb stick

func cmdUsb(args []string) error {
	root, ok := flagValue(args, "root")
	if !ok {
		return fmt.Errorf("--root fehlt")
	}
	if err := os.MkdirAll(root, 0o755); err != nil {
		return err
	}

	mac, hasMac := flagValue(args, "mac")
	win, hasWin := flagValue(args, "win")

	if hasMac {
		dest := filepath.Join(root, appName+".app")
		if err := copyTree(mac, dest); err != nil {
			return fmt.Errorf("macOS-App kopieren: %w", err)
		}
		fmt.Println("macOS:", filepath.Join(root, appName+".app"))
	}
	if hasWin {
		dest := filepath.Join(root, "macREC.exe")
		if err := copyFile(win, dest, 0o644); err != nil {
			return fmt.Errorf("Windows-Programm kopieren: %w", err)
		}
		fmt.Println("Windows:", dest)
	}

	readme := `macREC - Aufnahmesoftware (C) 2026 Niklas Legler - niklaslegler.at

macOS:   macREC - Aufnahmesoftware.app öffnen (Doppelklick). Beim ersten Start:
         Systemeinstellungen → Datenschutz & Sicherheit → Bildschirmaufnahme
         → "macREC" aktivieren.

Windows: macREC.exe starten. Beim ersten Start kann Windows fragen, ob die
         Datei wirklich ausgeführt werden soll: "Weitere Informationen" →
         "Trotzdem ausführen".

Die Screenshots landen automatisch in <Name>/screens/ auf diesem Stick.
`
	return os.WriteFile(filepath.Join(root, "README.txt"), []byte(readme), 0o644)
}

// ------------------------------------------------------------------ Helfer

func copyFile(source, target string, mode os.FileMode) error {
	data, err := os.ReadFile(source)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(target), 0o755); err != nil {
		return err
	}
	return os.WriteFile(target, data, mode)
}

func copyTree(source, target string) error {
	return filepath.Walk(source, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(source, path)
		if err != nil {
			return err
		}
		dest := filepath.Join(target, rel)
		if info.IsDir() {
			return os.MkdirAll(dest, 0o755)
		}
		mode := info.Mode()
		if mode&0o111 != 0 {
			mode |= 0o755
		}
		return copyFile(path, dest, mode)
	})
}
