// Utility for dealing with two-dimensional coordinate systems

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct XY {
    pub x: usize,
    pub y: usize
}

impl XY {

    pub fn new(x: usize, y: usize) -> XY {
        XY { x, y}
    }

    pub fn north(&self) -> Option<XY> {
        match self.y {
            0 => None,
            _ => Some(XY { x: self.x, y: self.y - 1 })
        }
    }

    pub fn south(&self) -> XY {
        XY { x: self.x, y: self.y + 1 }
    }

    pub fn west(&self) -> Option<XY> {
        match self.x {
            0 => None,
            _ => Some(XY { x: self.x - 1, y: self.y })
        }
    }

    pub fn east(&self) -> XY {
        XY { x: self.x + 1, y: self.y }
    }
}

#[cfg(test)]
mod coordinate_spec {
    use super::*;

    #[test]
    fn north_spec() {
        let xy: XY = XY { x: 5, y: 7};
        assert_eq!(xy.north(), Some(XY { x: 5, y: 6}));

        let xy: XY = XY { x: 5, y: 0 };
        assert_eq!(xy.north(), None);

        let xy: XY = XY { x: 0, y: 7 };
        assert_eq!(xy.north(), Some( XY { x: 0, y: 6 }));
    }

    #[test]
    fn south_spec() {
        let xy: XY = XY { x: 5, y: 7 };
        assert_eq!(xy.south(), XY { x: 5, y: 8 });

        let xy: XY = XY { x: 5, y: 0 };
        assert_eq!(xy.south(), XY { x: 5, y: 1 });

        let xy: XY = XY { x: 0, y: 7 };
        assert_eq!(xy.south(), XY { x: 0, y: 8 });
    }

    #[test]
    fn west_spec() {
        let xy: XY = XY { x: 5, y: 7};
        assert_eq!(xy.west(), Some(XY { x: 4, y: 7}));

        let xy: XY = XY { x: 5, y: 0 };
        assert_eq!(xy.west(), Some(XY { x: 4, y: 0 }));

        let xy: XY = XY { x: 0, y: 7 };
        assert_eq!(xy.west(), None);
    }

    #[test]
    fn east_spec() {
        let xy: XY = XY { x: 5, y: 7 };
        assert_eq!(xy.east(), XY { x: 6, y: 7 });

        let xy: XY = XY { x: 5, y: 0 };
        assert_eq!(xy.east(), XY { x: 6, y: 0 });

        let xy: XY = XY { x: 0, y: 7 };
        assert_eq!(xy.east(), XY { x: 1, y: 7 });
    }
}

