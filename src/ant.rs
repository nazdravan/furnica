//! Furnica desenată vectorial: se vede clar la orice mărime, se rotește lin
//! și își mișcă picioarele când merge (în trepied, ca furnicile adevărate).
//!
//! Desenul e făcut privind în sus, într-un pătrat cu latura 1 centrat în (0, 0);
//! `paint` îl rotește și îl scalează. Aceeași formă este în `assets/icon.png`.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::{Color32, Painter, Pos2, Shape, Stroke, vec2};

use crate::world::Dir;

#[derive(Clone, Copy)]
pub struct Colors {
    pub body: Color32,
    pub legs: Color32,
    pub shine: Color32,
    pub eyes: Color32,
}

impl Colors {
    /// Aceleași culori, mai transparente (de exemplu pentru previzualizare).
    pub fn faded(self, alpha: f32) -> Colors {
        Colors {
            body: self.body.gamma_multiply(alpha),
            legs: self.legs.gamma_multiply(alpha),
            shine: self.shine.gamma_multiply(alpha),
            eyes: self.eyes.gamma_multiply(alpha),
        }
    }
}

/// Unghiul (în radiani, în sensul acelor ceasornicului) față de „privește în sus”.
pub fn angle(d: Dir) -> f32 {
    match d {
        Dir::Up => 0.0,
        Dir::Right => FRAC_PI_2,
        Dir::Down => PI,
        Dir::Left => -FRAC_PI_2,
    }
}

/// Unghiul dintre `from` și `to` la momentul `t` (0..1), pe drumul cel mai scurt.
pub fn turn(from: f32, to: f32, t: f32) -> f32 {
    let mut d = (to - from).rem_euclid(TAU);
    if d > PI {
        d -= TAU;
    }
    from + d * t.clamp(0.0, 1.0)
}

/// Picioarele unei părți: (șold, genunchi, talpă), pentru partea dreaptă.
const LEGS: [[(f32, f32); 3]; 3] = [
    [(0.05, -0.11), (0.21, -0.19), (0.29, -0.35)],
    [(0.06, -0.06), (0.25, -0.06), (0.40, -0.01)],
    [(0.05, -0.01), (0.22, 0.07), (0.31, 0.27)],
];

/// Antena dreaptă: bază, cot, vârf.
const ANTENNA: [(f32, f32); 3] = [(0.035, -0.35), (0.11, -0.44), (0.20, -0.47)];

/// Desenează furnica cu centrul în `center`, încăpând într-un pătrat cu latura `size`.
/// `gait` crește cu 1 la fiecare pas; 0 = furnica stă.
pub fn paint(p: &Painter, center: Pos2, size: f32, angle: f32, gait: f32, c: Colors) {
    let (sin, cos) = angle.sin_cos();
    let at = |x: f32, y: f32| center + vec2(x * cos - y * sin, x * sin + y * cos) * size;
    let ellipse = |cx: f32, cy: f32, rx: f32, ry: f32| -> Vec<Pos2> {
        (0..28)
            .map(|i| {
                let t = i as f32 / 28.0 * TAU;
                at(cx + rx * t.cos(), cy + ry * t.sin())
            })
            .collect()
    };
    let width = (size * 0.045).max(1.2);
    let limb = |points: &[Pos2], color: Color32| {
        p.add(Shape::line(points.to_vec(), Stroke::new(width, color)));
        // Capete rotunjite.
        for &q in points {
            p.circle_filled(q, width / 2.0, color);
        }
    };

    // Umbra, puțin în jos indiferent de direcție.
    let shadow: Vec<Pos2> = ellipse(0.0, 0.02, 0.22, 0.40)
        .into_iter()
        .map(|q| q + vec2(size * 0.03, size * 0.05))
        .collect();
    p.add(Shape::convex_polygon(
        shadow,
        Color32::from_black_alpha((40.0 * c.body.a() as f32 / 255.0) as u8),
        Stroke::NONE,
    ));

    // Picioarele: trei pe o parte se mișcă împreună cu cele opuse lor de pe cealaltă.
    let swing = (gait * TAU).sin() * 0.06;
    for side in [-1.0, 1.0] {
        for (i, leg) in LEGS.iter().enumerate() {
            let group = if (i % 2 == 0) == (side > 0.0) {
                1.0
            } else {
                -1.0
            };
            let s = swing * group;
            let pts = [
                at(side * leg[0].0, leg[0].1),
                at(side * leg[1].0, leg[1].1 + s * 0.5),
                at(side * leg[2].0, leg[2].1 + s),
            ];
            limb(&pts, c.legs);
        }
        let wave = (gait * TAU * 2.0).sin() * 0.02 * side;
        let pts = ANTENNA.map(|(x, y)| at(side * x + wave, y));
        limb(&pts, c.legs);
        p.circle_filled(pts[2], width * 0.9, c.legs);
    }

    // Corpul: abdomen, mijloc, cap.
    p.add(Shape::convex_polygon(
        ellipse(0.0, 0.24, 0.16, 0.22),
        c.body,
        Stroke::NONE,
    ));
    p.circle_filled(at(0.0, 0.065), size * 0.04, c.body);
    p.add(Shape::convex_polygon(
        ellipse(0.0, -0.06, 0.075, 0.11),
        c.body,
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        ellipse(0.0, -0.27, 0.115, 0.10),
        c.body,
        Stroke::NONE,
    ));

    // Luciul și ochii.
    p.add(Shape::convex_polygon(
        ellipse(-0.055, 0.17, 0.04, 0.075),
        c.shine,
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        ellipse(-0.03, -0.09, 0.02, 0.04),
        c.shine,
        Stroke::NONE,
    ));
    for side in [-1.0, 1.0] {
        p.circle_filled(at(side * 0.06, -0.30), size * 0.028, c.eyes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_the_short_way() {
        let left = angle(Dir::Left);
        let up = angle(Dir::Up);
        assert!((turn(up, left, 0.5) + PI / 4.0).abs() < 1e-5);
        // De la stînga la jos: un sfert de tură în sens invers acelor, nu trei sferturi.
        let half = turn(left, angle(Dir::Down), 0.5);
        assert!((half - (-FRAC_PI_2 - PI / 4.0)).abs() < 1e-5, "{half}");
        assert_eq!(turn(up, up, 0.7), up);
    }
}
