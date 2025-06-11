use std::fmt;

// Use hsize as a reasonable smaller (and consistent) version of usize, which
// might be a u64 on some platforms. We'll never realistically get even close to
// 2^64 instances of any of the Id'ed types, so this is fine to do.
#[allow(non_camel_case_types)]
pub type hsize = u32;

pub trait Id: 'static + Copy + fmt::Debug + Eq + Ord {
    /// Create a id from the given index. The index must not be
    /// `hsize::MAX` as this value is reserved!
    fn new(idx: hsize) -> Self;

    /// Return the index of the current handle.
    fn idx(&self) -> hsize;

    /// Creates an invalid temporary ID, used when we are allocating items
    /// sequentially which relate to each other.
    #[inline(always)]
    fn temp() -> Self {
        Self::new(hsize::MAX)
    }

    /// Helper method to create a handle directly from an `usize`.
    ///
    /// If `raw` cannot be represented by `hsize`, this function either panics
    /// or returns a nonsensical ID. In debug mode, this function is guaranteed
    /// to panic in this case.
    #[inline(always)]
    fn from_usize(raw: usize) -> Self {
        // If `usize` is bigger than `hsize`, we assert that the value is fine.
        #[cfg(target_pointer_width = "64")]
        debug_assert!(raw <= hsize::MAX as usize);

        Self::new(raw as hsize)
    }

    /// Helper method to get the ID as a usize directly from an handle.
    ///
    /// If the index cannot be represented by `usize`, this function either
    /// panics or returns a nonsensical value. In debug mode, this function is
    /// guaranteed to panic in this case. Note however, that this usually won't
    /// happen, because `hsize` is in almost all cases smaller than or equal to
    /// `usize`.
    #[inline(always)]
    fn to_usize(&self) -> usize {
        // If `usize` is smaller than `hsize`, we assert that the value is fine.
        #[cfg(target_pointer_width = "16")]
        debug_assert!(self.idx() <= usize::MAX as hsize);

        self.idx() as usize
    }
}

macro_rules! make_handle_type {
    ($(#[$attr:meta])* $name:ident = $short:expr;) => {
        $(#[$attr])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(hsize);

        impl Id for $name {
            #[inline(always)]
            fn new(id: hsize) -> Self {
                $name(id)
            }

            #[inline(always)]
            fn idx(&self) -> hsize {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", $short)?;
                self.idx().fmt(f)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::temp()
            }
        }
    }
}

// Window things
make_handle_type! { ViewportId = "Viewport"; }
make_handle_type! { MaterialId = "Mat"; }
make_handle_type! { TextureId = "Tex"; }
// Mesh-specific handles
make_handle_type! { MeshId = "Mesh"; }
make_handle_type! { FaceId = "F"; }
make_handle_type! { EdgeId = "E"; }
make_handle_type! { VertexId = "V"; }
make_handle_type! { LoopId = "L"; }
make_handle_type! { PieceId = "P"; }
