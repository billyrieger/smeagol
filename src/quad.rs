#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quad<T> {
    pub nw: T,
    pub ne: T,
    pub sw: T,
    pub se: T,
}

impl<T> Quad<T> {
    pub fn from_array([nw, ne, sw, se]: [T; 4]) -> Self {
        Self { nw, ne, sw, se }
    }

    pub fn to_array(self) -> [T; 4] {
        [self.nw, self.ne, self.sw, self.se]
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Quad<U> {
        self.to_array().map(f).to_quad()
    }

    pub fn try_map<U>(self, f: impl FnMut(T) -> Option<U>) -> Option<Quad<U>> {
        Some(self.to_array().try_map(f)?.to_quad())
    }
}

impl<T> Quad<Quad<T>> {
    pub fn north(self) -> Quad<T> {
        Quad {
            nw: self.nw.ne,
            ne: self.ne.nw,
            sw: self.nw.se,
            se: self.ne.sw,
        }
    }

    pub fn south(self) -> Quad<T> {
        Quad {
            nw: self.sw.ne,
            ne: self.se.nw,
            sw: self.sw.se,
            se: self.se.sw,
        }
    }

    pub fn east(self) -> Quad<T> {
        Quad {
            nw: self.ne.sw,
            ne: self.ne.se,
            sw: self.se.nw,
            se: self.se.ne,
        }
    }

    pub fn west(self) -> Quad<T> {
        Quad {
            nw: self.nw.sw,
            ne: self.nw.se,
            sw: self.sw.nw,
            se: self.sw.ne,
        }
    }

    pub fn center(self) -> Quad<T> {
        Quad {
            nw: self.nw.se,
            ne: self.ne.sw,
            sw: self.sw.ne,
            se: self.se.nw,
        }
    }
}

pub trait ToQuad<T> {
    fn to_quad(self) -> Quad<T>;
}

impl<T> ToQuad<T> for [T; 4] {
    fn to_quad(self) -> Quad<T> {
        Quad::from_array(self)
    }
}