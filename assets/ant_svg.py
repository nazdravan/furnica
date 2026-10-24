# Furnica pentru pagina de prezentare (site/assets/): aceeași formă ca în
# src/ant.rs și în icon.py, ca SVG.
#   python3 assets/ant_svg.py
import math
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "site" / "assets"

LEGS = [[(0.05, -0.11), (0.21, -0.19), (0.29, -0.35)],
        [(0.06, -0.06), (0.25, -0.06), (0.40, -0.01)],
        [(0.05, -0.01), (0.22, 0.07), (0.31, 0.27)]]
ANTENNA = [(0.035, -0.35), (0.11, -0.44), (0.20, -0.47)]


def ant(angle, body, legs, eyes, shine):
    sn, cs = math.sin(angle), math.cos(angle)

    def at(x, y):
        return (50 + (x * cs - y * sn) * 100, 50 + (x * sn + y * cs) * 100)

    def pts(points):
        return " ".join(f"{x:.2f},{y:.2f}" for x, y in points)

    def ellipse(cx, cy, rx, ry, fill):
        p = [at(cx + rx * math.cos(t / 48 * math.tau), cy + ry * math.sin(t / 48 * math.tau)) for t in range(48)]
        return f'<polygon points="{pts(p)}" fill="{fill}"/>'

    w = 4.5
    out = []
    limb = f'fill="none" stroke="{legs}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"'
    for side in (-1, 1):
        for leg in LEGS:
            out.append(f'<polyline points="{pts(at(side * x, y) for x, y in leg)}" {limb}/>')
        a = [at(side * x, y) for x, y in ANTENNA]
        out.append(f'<polyline points="{pts(a)}" {limb}/>')
        out.append(f'<circle cx="{a[2][0]:.2f}" cy="{a[2][1]:.2f}" r="{w * 0.9:.2f}" fill="{legs}"/>')
    out.append(ellipse(0, 0.24, 0.16, 0.22, body))
    c = at(0, 0.065)
    out.append(f'<circle cx="{c[0]:.2f}" cy="{c[1]:.2f}" r="4" fill="{body}"/>')
    out.append(ellipse(0, -0.06, 0.075, 0.11, body))
    out.append(ellipse(0, -0.27, 0.115, 0.10, body))
    out.append(ellipse(-0.055, 0.17, 0.04, 0.075, shine))
    out.append(ellipse(-0.03, -0.09, 0.02, 0.04, shine))
    for side in (-1, 1):
        e = at(side * 0.06, -0.30)
        out.append(f'<circle cx="{e[0]:.2f}" cy="{e[1]:.2f}" r="2.8" fill="{eyes}"/>')
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="-6 -6 112 112" width="100" height="100">'
            + "".join(out) + "</svg>\n")


if __name__ == "__main__":
    # Culorile furnicii din tema luminoasă a programului (src/app.rs, Pal::ant).
    (OUT / "ant.svg").write_text(ant(0.5, "#B4481A", "#6E2A0C", "#FFF6EC", "rgba(255,255,255,0.27)"))
    # Pentru programul original: aceeași furnică, gri, întoarsă spre stânga.
    (OUT / "ant-old.svg").write_text(ant(-0.9, "#9A948C", "#77716A", "#F1F1EF", "rgba(255,255,255,0.2)"))
    print(OUT / "ant.svg", OUT / "ant-old.svg")
