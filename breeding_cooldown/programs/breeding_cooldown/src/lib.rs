use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod breeding_cooldown {
    use super::*;

    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;
        potion.authority = authority;
        // TODO: should price be here? should we charge to create the potion too?
        potion.price = 5;
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
        // TODO: optionally, do fast reaction

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    #[account(init, payer = user, space = 8 + 80)]
    pub potion: Account<'info, Potion>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct React<'info> {
    #[account(mut, has_one = authority)]
    pub potion: Account<'info, Potion>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Potion {
    pub authority: Pubkey,
    pub price: u64,
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
