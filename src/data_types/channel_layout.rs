//! Channel labels and layouts (`AudioChannelLayout`)

use objc2_core_audio_types::{kAudioChannelLabel_Center, kAudioChannelLabel_CenterSurround, kAudioChannelLabel_CenterTopFront, kAudioChannelLabel_CenterTopMiddle, kAudioChannelLabel_CenterTopRear, kAudioChannelLabel_Discrete_0, kAudioChannelLabel_LFE2, kAudioChannelLabel_LFEScreen, kAudioChannelLabel_Left, kAudioChannelLabel_LeftCenter, kAudioChannelLabel_LeftSideSurround, kAudioChannelLabel_LeftSurround, kAudioChannelLabel_LeftSurroundDirect, kAudioChannelLabel_LeftTopFront, kAudioChannelLabel_LeftTopMiddle, kAudioChannelLabel_LeftTopRear, kAudioChannelLabel_LeftWide, kAudioChannelLabel_RearSurroundLeft, kAudioChannelLabel_RearSurroundRight, kAudioChannelLabel_Right, kAudioChannelLabel_RightCenter, kAudioChannelLabel_RightSideSurround, kAudioChannelLabel_RightSurround, kAudioChannelLabel_RightSurroundDirect, kAudioChannelLabel_RightTopFront, kAudioChannelLabel_RightTopMiddle, kAudioChannelLabel_RightTopRear, kAudioChannelLabel_RightWide, kAudioChannelLabel_TopBackCenter, kAudioChannelLabel_TopBackLeft, kAudioChannelLabel_TopBackRight, kAudioChannelLabel_Unknown, kAudioChannelLabel_Unused, kAudioChannelLayoutTag_Atmos_5_1_2, kAudioChannelLayoutTag_Atmos_5_1_4, kAudioChannelLayoutTag_Atmos_7_1_2, kAudioChannelLayoutTag_Atmos_7_1_4, kAudioChannelLayoutTag_Atmos_9_1_6, kAudioChannelLayoutTag_Cube, kAudioChannelLayoutTag_DiscreteInOrder, kAudioChannelLayoutTag_Hexagonal, kAudioChannelLayoutTag_MPEG_5_0_A, kAudioChannelLayoutTag_MPEG_5_1_A, kAudioChannelLayoutTag_MPEG_6_1_A, kAudioChannelLayoutTag_MPEG_7_1_A, kAudioChannelLayoutTag_MPEG_7_1_C, kAudioChannelLayoutTag_Mono, kAudioChannelLayoutTag_Octagonal, kAudioChannelLayoutTag_Pentagonal, kAudioChannelLayoutTag_Quadraphonic, kAudioChannelLayoutTag_Stereo, kAudioChannelLayoutTag_Unknown, kAudioChannelLayoutTag_UseChannelBitmap, kAudioChannelLayoutTag_UseChannelDescriptions};
use crate::errors::{CoreAudioError, ErrorKind};

/// Size of the fixed `AudioChannelLayout` header: tag, bitmap and description
/// count
const LAYOUT_HEADER_SIZE: usize = 12;

/// Size of one `AudioChannelDescription`: label, flags and three coordinates
const DESCRIPTION_SIZE: usize = 20;

/// The speaker a channel feeds (`AudioChannelLabel`)
///
/// Wraps the raw label so any CoreAudio label can be carried, with the common
/// speaker positions available as constants
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelLabel(pub u32);

impl ChannelLabel {
    /// The channel's role is unknown
    pub const UNKNOWN: Self = Self(kAudioChannelLabel_Unknown);
    /// The channel is present but carries no signal
    pub const UNUSED: Self = Self(kAudioChannelLabel_Unused);
    /// Front left
    pub const LEFT: Self = Self(kAudioChannelLabel_Left);
    /// Front right
    pub const RIGHT: Self = Self(kAudioChannelLabel_Right);
    /// Front centre
    pub const CENTER: Self = Self(kAudioChannelLabel_Center);
    /// Low-frequency effects (the subwoofer)
    pub const LFE_SCREEN: Self = Self(kAudioChannelLabel_LFEScreen);
    /// Left surround
    pub const LEFT_SURROUND: Self = Self(kAudioChannelLabel_LeftSurround);
    /// Right surround
    pub const RIGHT_SURROUND: Self = Self(kAudioChannelLabel_RightSurround);
    /// Left of centre, between left and centre
    pub const LEFT_CENTER: Self = Self(kAudioChannelLabel_LeftCenter);
    /// Right of centre, between centre and right
    pub const RIGHT_CENTER: Self = Self(kAudioChannelLabel_RightCenter);
    /// Rear centre
    pub const CENTER_SURROUND: Self = Self(kAudioChannelLabel_CenterSurround);
    /// Left side surround, directly beside the listener
    pub const LEFT_SURROUND_DIRECT: Self = Self(kAudioChannelLabel_LeftSurroundDirect);
    /// Right side surround, directly beside the listener
    pub const RIGHT_SURROUND_DIRECT: Self = Self(kAudioChannelLabel_RightSurroundDirect);
    /// Left rear surround
    pub const REAR_SURROUND_LEFT: Self = Self(kAudioChannelLabel_RearSurroundLeft);
    /// Right rear surround
    pub const REAR_SURROUND_RIGHT: Self = Self(kAudioChannelLabel_RearSurroundRight);
    /// Left wide, between left and left surround
    pub const LEFT_WIDE: Self = Self(kAudioChannelLabel_LeftWide);
    /// Right wide, between right and right surround
    pub const RIGHT_WIDE: Self = Self(kAudioChannelLabel_RightWide);
    /// A second low-frequency effects channel
    pub const LFE2: Self = Self(kAudioChannelLabel_LFE2);
    /// Left top front (height)
    pub const LEFT_TOP_FRONT: Self = Self(kAudioChannelLabel_LeftTopFront);
    /// Centre top front (height)
    pub const CENTER_TOP_FRONT: Self = Self(kAudioChannelLabel_CenterTopFront);
    /// Right top front (height)
    pub const RIGHT_TOP_FRONT: Self = Self(kAudioChannelLabel_RightTopFront);
    /// Left top middle (height)
    pub const LEFT_TOP_MIDDLE: Self = Self(kAudioChannelLabel_LeftTopMiddle);
    /// Centre top middle (height, directly overhead)
    pub const CENTER_TOP_MIDDLE: Self = Self(kAudioChannelLabel_CenterTopMiddle);
    /// Right top middle (height)
    pub const RIGHT_TOP_MIDDLE: Self = Self(kAudioChannelLabel_RightTopMiddle);
    /// Left top rear (height)
    pub const LEFT_TOP_REAR: Self = Self(kAudioChannelLabel_LeftTopRear);
    /// Centre top rear (height)
    pub const CENTER_TOP_REAR: Self = Self(kAudioChannelLabel_CenterTopRear);
    /// Right top rear (height)
    pub const RIGHT_TOP_REAR: Self = Self(kAudioChannelLabel_RightTopRear);
    /// Left side surround
    pub const LEFT_SIDE_SURROUND: Self = Self(kAudioChannelLabel_LeftSideSurround);
    /// Right side surround
    pub const RIGHT_SIDE_SURROUND: Self = Self(kAudioChannelLabel_RightSideSurround);
    /// Older name for left top rear (height)
    pub const TOP_BACK_LEFT: Self = Self(kAudioChannelLabel_TopBackLeft);
    /// Older name for centre top rear (height)
    pub const TOP_BACK_CENTER: Self = Self(kAudioChannelLabel_TopBackCenter);
    /// Older name for right top rear (height)
    pub const TOP_BACK_RIGHT: Self = Self(kAudioChannelLabel_TopBackRight);

    /// A discrete channel with no speaker position, by zero-based index
    pub const fn discrete(index: u16) -> Self {
        Self(kAudioChannelLabel_Discrete_0 | index as u32)
    }
}

/// Identifies a predefined channel layout (`AudioChannelLayoutTag`)
///
/// The low 16 bits hold the channel count
/// [`USE_CHANNEL_DESCRIPTIONS`](Self::USE_CHANNEL_DESCRIPTIONS) means the
/// layout is given channel by channel instead
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelLayoutTag(pub u32);

impl ChannelLayoutTag {
    /// The layout is described by its channel descriptions
    pub const USE_CHANNEL_DESCRIPTIONS: Self = Self(kAudioChannelLayoutTag_UseChannelDescriptions);
    /// The layout is described by its channel bitmap
    pub const USE_CHANNEL_BITMAP: Self = Self(kAudioChannelLayoutTag_UseChannelBitmap);
    /// C
    pub const MONO: Self = Self(kAudioChannelLayoutTag_Mono);
    /// L R
    pub const STEREO: Self = Self(kAudioChannelLayoutTag_Stereo);
    /// Front left, front right, back left, back right
    pub const QUADRAPHONIC: Self = Self(kAudioChannelLayoutTag_Quadraphonic);
    /// Quadraphonic plus front centre
    pub const PENTAGONAL: Self = Self(kAudioChannelLayoutTag_Pentagonal);
    /// Pentagonal plus rear centre
    pub const HEXAGONAL: Self = Self(kAudioChannelLayoutTag_Hexagonal);
    /// Hexagonal plus side left and side right
    pub const OCTAGONAL: Self = Self(kAudioChannelLayoutTag_Octagonal);
    /// Eight speakers at the corners of a cube
    pub const CUBE: Self = Self(kAudioChannelLayoutTag_Cube);
    /// L R C Ls Rs
    pub const MPEG_5_0_A: Self = Self(kAudioChannelLayoutTag_MPEG_5_0_A);
    /// L R C LFE Ls Rs
    pub const MPEG_5_1_A: Self = Self(kAudioChannelLayoutTag_MPEG_5_1_A);
    /// L R C LFE Ls Rs Cs
    pub const MPEG_6_1_A: Self = Self(kAudioChannelLayoutTag_MPEG_6_1_A);
    /// L R C LFE Ls Rs Lc Rc
    pub const MPEG_7_1_A: Self = Self(kAudioChannelLayoutTag_MPEG_7_1_A);
    /// L R C LFE Ls Rs Rls Rrs
    pub const MPEG_7_1_C: Self = Self(kAudioChannelLayoutTag_MPEG_7_1_C);
    /// 5.1 with two top middle heights
    pub const ATMOS_5_1_2: Self = Self(kAudioChannelLayoutTag_Atmos_5_1_2);
    /// 5.1 with four heights, front and rear
    pub const ATMOS_5_1_4: Self = Self(kAudioChannelLayoutTag_Atmos_5_1_4);
    /// 7.1 with two top middle heights
    pub const ATMOS_7_1_2: Self = Self(kAudioChannelLayoutTag_Atmos_7_1_2);
    /// 7.1 with four heights, front and rear
    pub const ATMOS_7_1_4: Self = Self(kAudioChannelLayoutTag_Atmos_7_1_4);
    /// 9.1 with six heights, front, middle and rear
    pub const ATMOS_9_1_6: Self = Self(kAudioChannelLayoutTag_Atmos_9_1_6);
    /// Channels with no speaker positions; OR in the count
    pub const DISCRETE_IN_ORDER: Self = Self(kAudioChannelLayoutTag_DiscreteInOrder);
    /// An unknown layout; OR in the count
    pub const UNKNOWN: Self = Self(kAudioChannelLayoutTag_Unknown);

    /// The number of channels the tag describes, or `0` for the tags that
    /// defer to descriptions or a bitmap
    pub const fn channel_count(&self) -> u32 {
        self.0 & 0xFFFF
    }
}

/// One channel of a [`ChannelLayout`] (`AudioChannelDescription`)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelDescription {
    /// The speaker the channel feeds
    label: ChannelLabel,
    /// `AudioChannelFlags` describing the coordinates, if any
    flags: u32,
    /// Speaker coordinates, used when `label` is `kAudioChannelLabel_UseCoordinates`
    coordinates: [f32; 3],
}

impl From<ChannelLabel> for ChannelDescription {
    fn from(label: ChannelLabel) -> Self {
        Self {
            label,
            flags: 0,
            coordinates: [0.0; 3],
        }
    }
}

impl ChannelDescription {
    /// The speaker the channel feeds
    pub fn label(&self) -> ChannelLabel {
        self.label
    }

    /// `AudioChannelFlags` describing the coordinates
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Speaker coordinates, meaningful only when flags are set
    pub fn coordinates(&self) -> [f32; 3] {
        self.coordinates
    }

    /// Serialises the description as a native-endian `AudioChannelDescription`
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(DESCRIPTION_SIZE);
        bytes.extend_from_slice(&self.label.0.to_ne_bytes());
        bytes.extend_from_slice(&self.flags.to_ne_bytes());
        self.coordinates.iter().for_each(|c| bytes.extend_from_slice(&c.to_ne_bytes()));
        bytes
    }

    /// Reads a native-endian `AudioChannelDescription` from `bytes`
    fn from_bytes(bytes: &[u8]) -> Result<Self, CoreAudioError> {
        let word = |i: usize| -> Result<[u8; 4], CoreAudioError> {
            bytes.get(i * 4..i * 4 + 4)
                .and_then(|b| b.try_into().ok())
                .ok_or(CoreAudioError::from_error_kind(ErrorKind::ChannelLayoutConversion))
        };

        Ok(Self {
            label: ChannelLabel(u32::from_ne_bytes(word(0)?)),
            flags: u32::from_ne_bytes(word(1)?),
            coordinates: [
                f32::from_ne_bytes(word(2)?),
                f32::from_ne_bytes(word(3)?),
                f32::from_ne_bytes(word(4)?),
            ],
        })
    }
}

/// Which speaker each channel of a device or stream feeds (`AudioChannelLayout`)
///
/// A layout is either a predefined [`ChannelLayoutTag`], a channel bitmap, or
/// one [`ChannelDescription`] per channel
/// Use [`layout_for_tag`](crate::layout_for_tag) to expand a tag into its
/// descriptions
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelLayout {
    /// The predefined layout, or `USE_CHANNEL_DESCRIPTIONS` / `USE_CHANNEL_BITMAP`
    tag: ChannelLayoutTag,
    /// `AudioChannelBitmap`, used when `tag` is `USE_CHANNEL_BITMAP`
    bitmap: u32,
    /// One entry per channel, used when `tag` is `USE_CHANNEL_DESCRIPTIONS`
    descriptions: Vec<ChannelDescription>,
}

impl From<ChannelLayoutTag> for ChannelLayout {
    fn from(tag: ChannelLayoutTag) -> Self {
        Self {
            tag,
            bitmap: 0,
            descriptions: Vec::new(),
        }
    }
}

impl From<Vec<ChannelLabel>> for ChannelLayout {
    /// A layout given channel by channel, one label per channel
    fn from(labels: Vec<ChannelLabel>) -> Self {
        Self {
            tag: ChannelLayoutTag::USE_CHANNEL_DESCRIPTIONS,
            bitmap: 0,
            descriptions: labels.into_iter().map(ChannelDescription::from).collect(),
        }
    }
}

impl ChannelLayout {
    /// The predefined layout, or `USE_CHANNEL_DESCRIPTIONS` / `USE_CHANNEL_BITMAP`
    pub fn tag(&self) -> ChannelLayoutTag {
        self.tag
    }

    /// `AudioChannelBitmap`, used when the tag is `USE_CHANNEL_BITMAP`
    pub fn bitmap(&self) -> u32 {
        self.bitmap
    }

    /// One entry per channel, used when the tag is `USE_CHANNEL_DESCRIPTIONS`
    pub fn descriptions(&self) -> &[ChannelDescription] {
        &self.descriptions
    }

    /// The label of each channel, in channel order
    /// Empty unless the layout is given by descriptions
    pub fn labels(&self) -> Vec<ChannelLabel> {
        self.descriptions.iter().map(|d| d.label).collect()
    }

    /// Serialises the layout as a native-endian `AudioChannelLayout`
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(LAYOUT_HEADER_SIZE + self.descriptions.len() * DESCRIPTION_SIZE);
        bytes.extend_from_slice(&self.tag.0.to_ne_bytes());
        bytes.extend_from_slice(&self.bitmap.to_ne_bytes());
        bytes.extend_from_slice(&(self.descriptions.len() as u32).to_ne_bytes());
        self.descriptions.iter().for_each(|d| bytes.extend_from_slice(&d.to_bytes()));
        bytes
    }

    /// Reads a native-endian `AudioChannelLayout` from `bytes`, header and
    /// descriptions
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, CoreAudioError> {
        let error = || CoreAudioError::from_error_kind(ErrorKind::ChannelLayoutConversion);
        let word = |i: usize| -> Result<u32, CoreAudioError> {
            bytes.get(i * 4..i * 4 + 4)
                .and_then(|b| b.try_into().ok())
                .map(u32::from_ne_bytes)
                .ok_or(error())
        };

        let count = word(2)? as usize;
        let body = bytes.get(LAYOUT_HEADER_SIZE..LAYOUT_HEADER_SIZE + count * DESCRIPTION_SIZE).ok_or(error())?;

        Ok(Self {
            tag: ChannelLayoutTag(word(0)?),
            bitmap: word(1)?,
            descriptions: body.chunks_exact(DESCRIPTION_SIZE)
                .map(ChannelDescription::from_bytes)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_layout_round_trips() {
        let layout = ChannelLayout::from(vec![ChannelLabel::LEFT, ChannelLabel::RIGHT, ChannelLabel::UNUSED]);
        let bytes = layout.to_bytes();
        assert_eq!(bytes.len(), LAYOUT_HEADER_SIZE + 3 * DESCRIPTION_SIZE);
        assert_eq!(ChannelLayout::from_bytes(&bytes).unwrap(), layout);
    }

    #[test]
    fn short_channel_layout_is_rejected() {
        let mut bytes = ChannelLayout::from(vec![ChannelLabel::LEFT]).to_bytes();
        bytes.truncate(LAYOUT_HEADER_SIZE + 4);
        assert!(ChannelLayout::from_bytes(&bytes).is_err());
    }

    #[test]
    fn tag_channel_count() {
        assert_eq!(ChannelLayoutTag::ATMOS_9_1_6.channel_count(), 16);
        assert_eq!(ChannelLayoutTag::USE_CHANNEL_DESCRIPTIONS.channel_count(), 0);
    }
}
