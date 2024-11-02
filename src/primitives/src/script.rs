use super::*;

use sp_std::vec::Vec;

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
  Right,
  Left,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OpCode {
  // Specifies the title
  Title(Title),
  // Specifies the reference version that opcodes will be applied
  // until the next reference opcode in the script.
  Reference(ArticleVersion),
  // Puts the cursor to the beginning of the body data
  Begin,
  // Puts the cursor to the end of body data
  End,
  // Moves cursor `number` times in the direction
  MvCr(Direction, u32),
  // Adds the body data to the left starting from cursor position.
  // Final cursor position is the end of the data.
  Add(Body),
  // Deletes the `number` times of character from the right.
  // Final cursor position is last deleted character.
  Del(u32),
  // Copies “number” times of characters from the right. Cursor position doesn’t change.
  Cp(u32),
}
// NOTE: Default cursor position is the beginning of the body.

impl OpCode {
  pub fn requires_reference(&self) -> bool {
    !matches!(self, OpCode::Title(_) | OpCode::Reference(_) | OpCode::Add(_))
  }
}

pub const MAX_SCRIPT_LEN: u32 = 1_000;
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Script(
  #[cfg_attr(
    feature = "borsh",
    borsh(
      serialize_with = "borsh_serialize_bounded_vec",
      deserialize_with = "borsh_deserialize_bounded_vec"
    )
  )]
  BoundedVec<OpCode, ConstU32<{ MAX_SCRIPT_LEN }>>,
);

impl Script {
  pub fn new(data: Vec<OpCode>) -> Result<Script, &'static str> {
    Ok(Script(data.try_into().map_err(|_| "Script length exceeds {MAX_SCRIPT_LEN}")?))
  }

  pub fn data(&self) -> &[OpCode] {
    self.0.as_ref()
  }

  pub fn consume(self) -> Vec<OpCode> {
    self.0.into_inner()
  }
}

impl AsRef<[OpCode]> for Script {
  fn as_ref(&self) -> &[OpCode] {
    self.0.as_ref()
  }
}
