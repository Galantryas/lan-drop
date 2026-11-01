const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// 32x32 transparent PNG
const pngBase64 = "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAOxAAADsQBlSsOGwAAADhJREFUeJztwQEBAAAAgiD/r25IQAEAAPwGkAAAYC8GAACQkBAAAICFhAAAAAwhIQAAAAsICAAAAAA/sQB/4AYGvwAAAABJRU5ErkJggg==";
const pngBuf = Buffer.from(pngBase64, 'base64');

fs.writeFileSync(path.join(iconsDir, '32x32.png'), pngBuf);
fs.writeFileSync(path.join(iconsDir, '128x128.png'), pngBuf);
fs.writeFileSync(path.join(iconsDir, 'icon.png'), pngBuf);

// Create valid ICO file containing PNG payload
const icoHeader = Buffer.from([
  0x00, 0x00, // Reserved
  0x01, 0x00, // Image type: 1 = ICO
  0x01, 0x00, // Number of images: 1
  
  // Icon directory entry (16 bytes)
  0x20, // Width 32
  0x20, // Height 32
  0x00, // Color palette count
  0x00, // Reserved
  0x01, 0x00, // Color planes
  0x20, 0x00, // Bits per pixel (32)
  
  // Size of image data (4 bytes uint32 LE)
  pngBuf.length & 0xff,
  (pngBuf.length >> 8) & 0xff,
  (pngBuf.length >> 16) & 0xff,
  (pngBuf.length >> 24) & 0xff,
  
  // Offset of image data (4 bytes uint32 LE = 22 = 0x16)
  0x16, 0x00, 0x00, 0x00
]);

const icoBuf = Buffer.concat([icoHeader, pngBuf]);
fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoBuf);

console.log('Icons generated successfully in src-tauri/icons');
