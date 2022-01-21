use anchor_lang::prelude::*;

declare_id!("G23nNQ3tMzXaT4SQecJfW6JSwuMhsBruU9iZvhPFeehu");

const PREFIX: &str = "bapeBreedingTest17";
// const PREFIX_POTION: &str = "potion";

#[program]
pub mod breeding_state {
    use super::*;

    pub fn create_potion_state(ctx: Context<CreatePotionState>) -> ProgramResult {
        let user = &ctx.accounts.user;
        let potion_state = &mut ctx.accounts.potion_state;

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);
        let nft_1_state = &mut ctx.accounts.nft_1_state;
        if nft_1_state.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }
        let nft_2_state = &mut ctx.accounts.nft_2_state;
        if nft_2_state.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        // set metadata
        let nft_1_key = *ctx.accounts.nft_1.key;
        let nft_2_key = *ctx.accounts.nft_2.key;
        potion_state.authority = *user.key;
        potion_state.nft1 = nft_1_key;
        potion_state.nft2 = nft_2_key;
        potion_state.created_timestamp = timestamp;

        nft_1_state.authority = *user.key;
        nft_1_state.nft = nft_1_key;
        nft_1_state.last_bred_timestamp = timestamp;
        nft_2_state.authority = *user.key;
        nft_2_state.nft = nft_2_key;
        nft_2_state.last_bred_timestamp = timestamp;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePotionState<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub potion_mint: AccountInfo<'info>,
    #[account(init, payer = user, space = 8 + 120, seeds = [PREFIX.as_ref(), potion_mint.key.as_ref()], bump)]
    pub potion_state: Account<'info, PotionState>,

    // TODO: come back for validations
    // #[account(owner = *user.key)]
    pub nft_1: AccountInfo<'info>,
    // constraint= config.to_account_info().owner
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_1.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_1_state: Account<'info, NftState>,
    // #[account(owner = *user.key)]
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [PREFIX.as_bytes(), nft_2.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_2_state: Account<'info, NftState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct PotionState {
    pub authority: Pubkey,
    pub nft1: Pubkey,
    pub nft2: Pubkey,
    pub created_timestamp: u64
}

#[account]
#[derive(Default)]
pub struct NftState {
    pub authority: Pubkey,
    pub nft: Pubkey,
    pub last_bred_timestamp: u64
}

#[error]
pub enum ErrorCode {
    #[msg("This NFT has been used for breeding in the last 10 days.")]
    NftUsedTooSoon,
}

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let seven_days_in_seconds = 7 * 24 * 60 * 60;
    return timestamp - seven_days_in_seconds;
}
