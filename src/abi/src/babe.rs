use sp_consensus_babe::EquivocationProof;

use wikiblocks_primitives::{Header, WikiblocksAddress};

#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub struct ReportEquivocation {
  pub equivocation_proof: alloc::boxed::Box<EquivocationProof<Header>>,
  pub key_owner_proof: WikiblocksAddress,
}

// We could define a Babe Config here and use the literal pallet_babe::Call
// The disadvantage to this would be the complexity and presence of junk fields such as `__Ignore`
#[derive(Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum Call {
  report_equivocation(ReportEquivocation),
  report_equivocation_unsigned(ReportEquivocation),
}
