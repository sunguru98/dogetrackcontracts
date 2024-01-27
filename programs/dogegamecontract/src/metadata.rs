use std::ops::Deref;

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use mpl_token_metadata::{
    state::{Metadata as TokenMetadata, MAX_METADATA_LEN},
    ID,
};

#[derive(Clone)]
pub struct Metadata(TokenMetadata);

impl Metadata {
    pub const LEN: usize = MAX_METADATA_LEN;
}

impl anchor_lang::AccountDeserialize for Metadata {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        TokenMetadata::deserialize(buf)
            .map(Metadata)
            .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for Metadata {}

impl anchor_lang::Owner for Metadata {
    fn owner() -> Pubkey {
        ID
    }
}

impl Deref for Metadata {
    type Target = TokenMetadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
