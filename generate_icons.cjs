const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// Valid 32x32 PNG (blue icon)
const pngBase64 = "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAAUSURBVFhH7cExAQAAAMKg9U9tCjUAAIAbD20AAXXv0QkAAAAASUVORK5CYII=";
const pngBuf = Buffer.from(pngBase64, 'base64');

fs.writeFileSync(path.join(iconsDir, '32x32.png'), pngBuf);
fs.writeFileSync(path.join(iconsDir, '128x128.png'), pngBuf);
fs.writeFileSync(path.join(iconsDir, 'icon.png'), pngBuf);

// Create valid 32x32 ICO file with BMP header
const header = Buffer.alloc(6 + 16);
header.writeUInt16LE(0, 0); // Reserved
header.writeUInt16LE(1, 2); // Type = ICO
header.writeUInt16LE(1, 4); // Count = 1

// Entry
header.writeUInt8(32, 6); // Width
header.writeUInt8(32, 7); // Height
header.writeUInt8(0, 8);  // Palette
header.writeUInt8(0, 9);  // Reserved
header.writeUInt16LE(1, 10); // Planes
header.writeUInt16LE(32, 12); // BPP
const dataSize = 40 + (32 * 32 * 4) + (32 * 4); // Header + XOR + AND
header.writeUInt32LE(dataSize, 14);
header.writeUInt32LE(22, 18); // Offset

const bmpHeader = Buffer.alloc(40);
bmpHeader.writeUInt32LE(40, 0); // Header size
bmpHeader.writeInt32LE(32, 4);  // Width
bmpHeader.writeInt32LE(64, 8);  // Height (double for ICO)
bmpHeader.writeUInt16LE(1, 12); // Planes
bmpHeader.writeUInt16LE(32, 14); // BPP
bmpHeader.writeUInt32LE(0, 16);  // Compression
bmpHeader.writeUInt32LE(32 * 32 * 4, 20); // Image size

const xorMask = Buffer.alloc(32 * 32 * 4);
for (let i = 0; i < 32 * 32 * 4; i += 4) {
  xorMask[i] = 0xf6;     // B
  xorMask[i + 1] = 0x82; // G
  xorMask[i + 2] = 0x3b; // R
  xorMask[i + 3] = 0xff; // A
}

const andMask = Buffer.alloc(32 * 4, 0); // All zeroes = opaque

const icoBuf = Buffer.concat([header, bmpHeader, xorMask, andMask]);
fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoBuf);

console.log('Regenerated valid ICO & PNG icons');
