use anchor_lang::prelude::*;

pub mod nft_storage;
pub mod vault;

use nft_storage::*;
use vault::*;

declare_id!("925EaFt3RoSBrjMcWp4YWh2muDziCtG1mNfQC152fSzx");

#[program]
pub mod nft_vault_swap {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        vault::initialize_vault(ctx)
    }

    pub fn lock_nft(ctx: Context<LockNFT>, lock_duration: i64) -> Result<()> {
        vault::lock_nft(ctx, lock_duration)
    }

    pub fn unlock_nft(ctx: Context<UnlockNFT>) -> Result<()> {
        vault::unlock_nft(ctx)
    }

    pub fn withdraw_rent(ctx: Context<WithdrawRent>, amount: u64) -> Result<()> {
        vault::withdraw_rent(ctx, amount)
    }

    pub fn create_nft(
        ctx: Context<CreateNFT>,
        name: String,
        symbol: String,
        uri: String,
        image_cid: String,
    ) -> Result<()> {
        nft_storage::create_nft(ctx, name, symbol, uri, image_cid)
    }

    pub fn update_nft_data(
        ctx: Context<UpdateNFTData>,
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
        image_cid: Option<String>,
    ) -> Result<()> {
        nft_storage::update_nft_data(ctx, name, symbol, uri, image_cid)
    }
}
