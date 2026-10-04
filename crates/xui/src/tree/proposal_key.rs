use crate::SizeProposal;

// A size proposal as hashable bits, to cache the size a node answered for it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ProposalKey {
    width: Option<u32>,
    height: Option<u32>,
}

impl From<SizeProposal> for ProposalKey {
    fn from(proposal: SizeProposal) -> ProposalKey {
        ProposalKey {
            width: proposal.width.map(f32::to_bits),
            height: proposal.height.map(f32::to_bits),
        }
    }
}
