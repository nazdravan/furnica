# Iconița Furnicii: aceeași formă ca în src/ant.rs, pe insigna portocalie din siglă.
import math, sys
from PIL import Image, ImageDraw
out = sys.argv[1]
N = 1024  # desenat mare, apoi micșorat
img = Image.new('RGBA', (N, N), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
r = int(N * 0.22)
# insigna cu degrade vertical
grad = Image.new('RGBA', (N, N))
for y in range(N):
    t = y / N
    c = tuple(int(a + (b - a) * t) for a, b in zip((0xF5, 0x9E, 0x2B), (0xDC, 0x5F, 0x16)))
    ImageDraw.Draw(grad).line([(0, y), (N, y)], fill=c + (255,))
mask = Image.new('L', (N, N), 0)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, N - 1, N - 1], r, fill=255)
img.paste(grad, (0, 0), mask)

size, angle = N * 0.80, 0.5
cx = cy = N / 2
sn, cs = math.sin(angle), math.cos(angle)
def at(x, y):
    return (cx + (x * cs - y * sn) * size, cy + (x * sn + y * cs) * size)
def ellipse(x, y, rx, ry, fill):
    d.polygon([at(x + rx * math.cos(t / 64 * 2 * math.pi), y + ry * math.sin(t / 64 * 2 * math.pi)) for t in range(64)], fill=fill)
W = size * 0.05
def limb(pts, fill):
    d.line(pts, fill=fill, width=int(W), joint='curve')
    for p in pts:
        d.ellipse([p[0] - W / 2, p[1] - W / 2, p[0] + W / 2, p[1] + W / 2], fill=fill)
cream, orange = (0xFF, 0xF6, 0xEC, 255), (0xE8, 0x74, 0x22, 255)
LEGS = [[(0.05, -0.11), (0.21, -0.19), (0.29, -0.35)],
        [(0.06, -0.06), (0.25, -0.06), (0.40, -0.01)],
        [(0.05, -0.01), (0.22, 0.07), (0.31, 0.27)]]
ANT = [(0.035, -0.35), (0.11, -0.44), (0.20, -0.47)]
for side in (-1, 1):
    for leg in LEGS:
        limb([at(side * x, y) for x, y in leg], cream)
    pts = [at(side * x, y) for x, y in ANT]
    limb(pts, cream)
    p = pts[2]; rr = W * 0.9
    d.ellipse([p[0] - rr, p[1] - rr, p[0] + rr, p[1] + rr], fill=cream)
ellipse(0, 0.24, 0.16, 0.22, cream)
ellipse(0, 0.065, 0.04, 0.04, cream)
ellipse(0, -0.06, 0.075, 0.11, cream)
ellipse(0, -0.27, 0.115, 0.10, cream)
for side in (-1, 1):
    ellipse(side * 0.06, -0.30, 0.028, 0.028, orange)

icon = img.resize((256, 256), Image.LANCZOS)
icon.save(f'{out}/icon.png', optimize=True)
icon.save(f'{out}/furnica.ico', sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
