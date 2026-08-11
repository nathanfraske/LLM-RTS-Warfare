//! Square-grid indexing and neighborhood iteration.

/// Square cell grid of side `size`; cells indexed row-major.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    pub size: u32,
}

impl Grid {
    #[must_use]
    pub fn cells(self) -> usize {
        (self.size as usize) * (self.size as usize)
    }

    #[must_use]
    pub fn idx(self, x: u32, y: u32) -> usize {
        (y as usize) * (self.size as usize) + (x as usize)
    }

    #[must_use]
    pub fn xy(self, i: usize) -> (u32, u32) {
        (
            (i % self.size as usize) as u32,
            (i / self.size as usize) as u32,
        )
    }

    #[must_use]
    pub fn on_border(self, i: usize) -> bool {
        let (x, y) = self.xy(i);
        x == 0 || y == 0 || x == self.size - 1 || y == self.size - 1
    }

    /// The up-to-eight neighbors of `i`, in a fixed deterministic order.
    #[must_use]
    pub fn neighbors8(self, i: usize) -> ([usize; 8], usize) {
        const OFFSETS: [(i64, i64); 8] = [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ];
        let (x, y) = self.xy(i);
        let mut out = [0usize; 8];
        let mut n = 0;
        for (dx, dy) in OFFSETS {
            let nx = i64::from(x) + dx;
            let ny = i64::from(y) + dy;
            if nx >= 0 && ny >= 0 && nx < i64::from(self.size) && ny < i64::from(self.size) {
                out[n] = self.idx(nx as u32, ny as u32);
                n += 1;
            }
        }
        (out, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_have_three_neighbors_and_interiors_eight() {
        let g = Grid { size: 8 };
        assert_eq!(g.neighbors8(0).1, 3);
        assert_eq!(g.neighbors8(g.idx(3, 3)).1, 8);
        assert!(g.on_border(g.idx(7, 3)));
        assert!(!g.on_border(g.idx(6, 3)));
    }
}
