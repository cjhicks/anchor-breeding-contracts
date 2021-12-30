use anchor_lang::prelude::*;
use spl_token;
use solana_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod breeding_cooldown {
    use super::*;

    /*
    Inputs:
    - Potion Account
    - Payer Account (also signer)
    - Payer Bape Wallet (account + token)??
    - $BAPE token program (owner of mint)
    - $BAPE token mint

    Steps:

    Outputs:
    - Created Potion Account from token (owned by program)
    - Burned xxx BAPE (5% inflation per 30 days?)
    */
    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;
        let bape_token_program = &ctx.accounts.bape_token_program;
        let user = &ctx.accounts.user;
        let user_bape_wallet = &ctx.accounts.user_bape_wallet;
        let bape_token_mint = &ctx.accounts.bape_token_mint;

        potion.authority = authority;

        // TODO: check that user has enough $BAPE to get a potion
        let burn_price = 350; // TODO: inflation?

        let burn_bape = spl_token::instruction::burn(
            bape_token_program.key,
            user_bape_wallet.key,
            bape_token_mint.key,
            user.key,
            &[&user.key],
            burn_price
        )?;
        solana_program::program::invoke(
            &burn_bape,
            &[
                user_bape_wallet.to_account_info(),
                user.to_account_info(),
                bape_token_program.to_account_info(),
            ]
        )?;

        // TODO: mark apes as "used" (need another state account for this?)

        // TODO: mint new NFT for potion

        // TODO: should price be here? should we charge to create the potion too?
        potion.fast_react_price = 5;
        potion.cooldown_days = 7; // days, could be parameterized
        let timestamp = Clock::get()?.unix_timestamp as u64;
        potion.created_timestamp = timestamp;

        Ok(())
    }

    pub fn react(ctx: Context<React>) -> ProgramResult {

        let potion = &mut ctx.accounts.potion;

        let day_to_seconds = 60 * 60 * 24;
        let cooldown_expiration = potion.created_timestamp + (potion.cooldown_days * day_to_seconds);
        let timestamp = Clock::get()?.unix_timestamp as u64;

        if cooldown_expiration > timestamp {
            return Err(ErrorCode::CooldownNotReached.into());
        }

        // TODO: mint new NFT

        Ok(())
    }

    pub fn fast_react(ctx: Context<FastReact>) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;

        // TODO: do fast reaction (burn more $BAPE?)
        // TODO: check that user has enough $BAPE to get a fast reaction
        let fast_burn_price = 175; // TODO: inflation?

        // TODO: mint new NFT

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    #[account(init, payer = user, space = 8 + 80)]
    pub potion: Account<'info, Potion>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_bape_wallet: Signer<'info>,
    pub bape_token_program: Program<'info, System>,
    pub bape_token_mint: Program<'info, System>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct React<'info> {
    #[account(mut, has_one = authority)]
    pub potion: Account<'info, Potion>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct FastReact<'info> {
    #[account(mut, has_one = authority)]
    pub potion: Account<'info, Potion>,
    pub authority: Signer<'info>,
    pub bape_token_program: Program<'info, System>,
    pub bape_token_mint: Program<'info, System>,
}

#[account]
pub struct Potion {
    pub authority: Pubkey,
    pub fast_react_price: u64,
    pub cooldown_days: u64,
    pub created_timestamp: u64
}

#[error]
pub enum ErrorCode {
    #[msg("This potion has already been created.")]
    AlreadyCreated,
    #[msg("This potion has not reached its cooldown period.")]
    CooldownNotReached,
}
