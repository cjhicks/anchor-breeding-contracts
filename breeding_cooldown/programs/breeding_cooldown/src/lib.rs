use anchor_lang::prelude::*;
use anchor_spl;
use spl_token;
use solana_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod breeding_cooldown {
    use super::*;

    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let user_token_account = ctx.accounts.user_token_account.to_account_info();
        let token_mint = ctx.accounts.token_mint.to_account_info();

        potion.authority = authority;

        // TODO: mark apes as "used" (need another state account for this?)
        // TODO: check if 7 days since done this...
        // Do we have an existing token metadata account for the ape?
        // If so, update with timestamp (need to handle secondary sale case though)
        // Otherwise, create before coming into this method?
        // Like this? https://github.com/LexBrill/SolNFT/blob/master/src/processor.rs#L72

        // TODO: check that user has enough $BAPE to get a potion
        let burn_price = 350; // TODO: inflation?
        let cpi_accounts = anchor_spl::token::Burn {
            to: user_token_account,
            mint: token_mint,
            authority: user.to_account_info(),
        };
        let cpi_program = token_program.clone();
        let burn_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::burn(burn_ctx, burn_price).expect("burn failed.");

        // TODO: mint new NFT for potion
        // anchor.web3.SystemProgram.createAccount({
        //     fromPubkey: walletKey,
        //     newAccountPubkey: eggMint,
        //     space: MintLayout.span,
        //     lamports: rent,
        //     programId: TOKEN_PROGRAM_ID,
        //   }),
        //   Token.createInitMintInstruction(
        //     TOKEN_PROGRAM_ID,
        //     eggMint,
        //     0,
        //     walletKey,
        //     walletKey
        //   ),
        //   createAssociatedTokenAccountInstruction(
        //     eggToken,
        //     walletKey,
        //     walletKey,
        //     eggMint
        //   ),
        //   Token.createMintToInstruction(
        //     TOKEN_PROGRAM_ID,
        //     eggMint,
        //     eggToken,
        //     walletKey,
        //     [],
        //     1
        //   ),
        // anchor_spl::token::mint_to(), amount: u64)

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
    pub user_token_account: AccountInfo<'info>,  // User's $BAPE (or generic token) account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
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
    #[account(mut)]
    pub user_token_account: Signer<'info>,  // User's $BAPE (or generic token) account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
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
