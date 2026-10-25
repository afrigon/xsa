use std::mem;

use ash::vk::native::{
    StdVideoEncodeH265PictureInfo, StdVideoEncodeH265ReferenceInfo, StdVideoEncodeH265ReferenceListsInfo,
    StdVideoEncodeH265SliceSegmentHeader, StdVideoH265ShortTermRefPicSet,
    StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_I, StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_P,
};

use crate::reference_picture::ReferencePicture;

// H.265: the slice header may name up to five merge candidates, sent as five minus the count.
const MAX_MERGE_CANDIDATES: u8 = 5;
// Std reference lists mark unused entries with this index.
const NO_REFERENCE_PICTURE: u8 = 0xFF;

// The H.265 syntax describing one frame to the encoder: its slice, the picture it predicts from and the reference
// information of both slots. The Std structures start zeroed, every flag and pointer unset; only the fields that
// matter are set.
pub(crate) struct PictureSyntax {
    pub slice_header: StdVideoEncodeH265SliceSegmentHeader,
    pub written_reference: StdVideoEncodeH265ReferenceInfo,
    pub reference: StdVideoEncodeH265ReferenceInfo,
    lists: StdVideoEncodeH265ReferenceListsInfo,
    reference_set: StdVideoH265ShortTermRefPicSet,
    written: ReferencePicture,
    predicted: bool,
}

impl PictureSyntax {
    // A frame written to `written`'s slot, predicted from `reference`, or a keyframe without one.
    pub fn new(written: &ReferencePicture, reference: Option<&ReferencePicture>) -> PictureSyntax {
        let mut syntax = PictureSyntax {
            slice_header: unsafe { mem::zeroed() },
            written_reference: unsafe { mem::zeroed() },
            reference: unsafe { mem::zeroed() },
            lists: unsafe { mem::zeroed() },
            reference_set: unsafe { mem::zeroed() },
            written: *written,
            predicted: reference.is_some(),
        };
        syntax.slice_header.MaxNumMergeCand = MAX_MERGE_CANDIDATES;
        syntax.written_reference.pic_type = written.picture_type;
        syntax.written_reference.PicOrderCntVal = written.order;
        syntax.lists.RefPicList0 = [NO_REFERENCE_PICTURE; 15];
        syntax.lists.RefPicList1 = [NO_REFERENCE_PICTURE; 15];

        match reference {
            Some(reference) => {
                syntax.slice_header.slice_type = StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_P;
                syntax.reference.pic_type = reference.picture_type;
                syntax.reference.PicOrderCntVal = reference.order;
                syntax.lists.RefPicList0[0] = reference.slot as u8;
                // The previous picture, one step back in output order.
                syntax.reference_set.num_negative_pics = 1;
                syntax.reference_set.used_by_curr_pic_s0_flag = 1;
            }
            None => syntax.slice_header.slice_type = StdVideoH265SliceType_STD_VIDEO_H265_SLICE_TYPE_I,
        }

        syntax
    }

    // The picture's description, pointing into this syntax, which must outlive it.
    pub fn picture(&self) -> StdVideoEncodeH265PictureInfo {
        let mut picture: StdVideoEncodeH265PictureInfo = unsafe { mem::zeroed() };
        picture.pic_type = self.written.picture_type;
        picture.PicOrderCntVal = self.written.order;
        picture.flags.set_is_reference(1);
        picture.flags.set_pic_output_flag(1);
        picture.pShortTermRefPicSet = &self.reference_set;

        if self.predicted {
            picture.pRefLists = &self.lists;
        } else {
            picture.flags.set_IrapPicFlag(1);
        }

        picture
    }
}
