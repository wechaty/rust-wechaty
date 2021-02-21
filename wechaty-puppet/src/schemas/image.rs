use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, FromPrimitive, ToPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum ImageType {
    Unknown,
    Thumbnail,
    HD,
    Artwork,
}
