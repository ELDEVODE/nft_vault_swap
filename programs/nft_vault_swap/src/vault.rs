use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("925EaFt3RoSBrjMcWp4YWh2muDziCtG1mNfQC152fSzx");

#[program]
pub mod nft_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.total_rent_collected = 0;
        Ok(())
    }

    pub fn lock_nft(ctx: Context<LockNFT>, lock_duration: i64) -> Result<()> {
        // Transfer NFT from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        // Create lock account
        let lock_account = &mut ctx.accounts.lock_account;
        lock_account.user = ctx.accounts.user.key();
        lock_account.mint = ctx.accounts.user_token_account.mint;
        lock_account.lock_time = Clock::get()?.unix_timestamp;
        lock_account.unlock_time = lock_account.lock_time + lock_duration;
        lock_account.rent_per_day = 10_000_000; // 0.01 SOL per day
        lock_account.bump = *ctx.bumps.get("lock_account").unwrap();

        emit!(NFTLocked {
            user: ctx.accounts.user.key(),
            mint: ctx.accounts.user_token_account.mint,
            lock_time: lock_account.lock_time,
            unlock_time: lock_account.unlock_time,
        });

        Ok(())
    }

    pub fn unlock_nft(ctx: Context<UnlockNFT>) -> Result<()> {
        let lock_account = &ctx.accounts.lock_account;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= lock_account.unlock_time,
            VaultError::NFTStillLocked
        );

        // Calculate rent
        let time_locked = current_time - lock_account.lock_time;
        let days_locked = time_locked / 86400 + 1; // Round up to nearest day
        let rent_amount = (days_locked as u64) * lock_account.rent_per_day;

        // Transfer rent from user to vault
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_lang::system_program::transfer(cpi_ctx, rent_amount)?;

        // Update vault's total rent collected
        let vault = &mut ctx.accounts.vault;
        vault.total_rent_collected = vault.total_rent_collected.checked_add(rent_amount).unwrap();

        // Transfer NFT back to user
        let seeds = &[
            b"nft-lock",
            lock_account.user.as_ref(),
            lock_account.mint.as_ref(),
            &[lock_account.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.lock_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, 1)?;

        emit!(NFTUnlocked {
            user: ctx.accounts.user.key(),
            mint: lock_account.mint,
            unlock_time: current_time,
            rent_paid: rent_amount,
        });

        Ok(())
    }

    pub fn withdraw_rent(ctx: Context<WithdrawRent>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(
            amount <= vault.total_rent_collected,
            VaultError::InsufficientFunds
        );

        let seeds = &[b"vault", &[vault.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        vault.total_rent_collected = vault.total_rent_collected.checked_sub(amount).unwrap();

        emit!(RentWithdrawn {
            authority: ctx.accounts.authority.key(),
            amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 1,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"nft-lock", user.key().as_ref(), user_token_account.mint.as_ref()],
        bump
    )]
    pub lock_account: Account<'info, LockAccount>,
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnlockNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        close = user,
        seeds = [b"nft-lock", user.key().as_ref(), lock_account.mint.as_ref()],
        bump = lock_account.bump,
        has_one = user,
    )]
    pub lock_account: Account<'info, LockAccount>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawRent<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        constraint = authority.key() == vault.authority
    )]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub total_rent_collected: u64,
    pub bump: u8,
}

#[account]
pub struct LockAccount {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub lock_time: i64,
    pub unlock_time: i64,
    pub rent_per_day: u64,
    pub bump: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("NFT is still locked")]
    NFTStillLocked,
    #[msg("Insufficient funds in the vault")]
    InsufficientFunds,
}

#[event]
pub struct NFTLocked {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub lock_time: i64,
    pub unlock_time: i64,
}

#[event]
pub struct NFTUnlocked {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub unlock_time: i64,
    pub rent_paid: u64,
}

#[event]
pub struct RentWithdrawn {
    pub authority: Pubkey,
    pub amount: u64,
}
