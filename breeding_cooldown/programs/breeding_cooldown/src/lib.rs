use anchor_lang::prelude::*;
use anchor_spl;
use spl_token;
use solana_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod breeding_cooldown {
    use super::*;

    /*
    This function is equivalent to breeding an egg: https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5
    */
    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_user_account = ctx.accounts.token_user_account.to_account_info();
        let token_mint = ctx.accounts.token_mint.to_account_info();
        let potion_mint = ctx.accounts.potion_mint.to_account_info();
        let rent = &ctx.accounts.rent;
        let system_program = &ctx.accounts.system_program.to_account_info();

        potion.authority = authority;

        // TODO: mark apes as "used" (need another state account for this?)
        // TODO: check if 7 days since done this...
        // Do we have an existing token metadata account for the ape?
        // If so, update with timestamp (need to handle secondary sale case though)
        // Otherwise, create before coming into this method?
        // Like this? https://github.com/LexBrill/SolNFT/blob/master/src/processor.rs#L72

        // TODO: check that user has enough $BAPE to get a potion

        /*
        Burn $BAPE before minting potion
        */
        let burn_price = 350; // TODO: inflation?
        let burn_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::Burn {
                to: token_user_account,
                mint: token_mint,
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, burn_price)
            .expect("burn failed.");

        /* 
        Mint new NFT for potion
        */

        // TODO: I think anchor does this automatically with account(init, ...)
        // anchor.web3.SystemProgram.createAccount({fromPubkey: walletKey, newAccountPubkey: potionMint, space: MintLayout.span, lamports: rent, programId: TOKEN_PROGRAM_ID,}),

        // Token.createInitMintInstruction(TOKEN_PROGRAM_ID, potion_mint, 0, walletKey, walletKey)
        let init_mint_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::InitializeMint {
                mint: potion_mint.to_account_info(),
                rent: rent.to_account_info()
            }
        );
        anchor_spl::token::initialize_mint(init_mint_ctx, 0, &user.key(), Some(&user.key()))
            .expect("Init Mint failed.");

        //   createAssociatedTokenAccountInstruction(potionToken, walletKey, walletKey, potionMint),
        let create_associated_token_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::associated_token::Create {
                payer: user.to_account_info(),
                associated_token: potion.to_account_info(),
                authority: user.to_account_info(),
                mint: potion_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: rent.clone(),
            }
        );
        anchor_spl::associated_token::create(create_associated_token_ctx)
            .expect("Create Associated Token failed.");

        //   Token.createMintToInstruction(TOKEN_PROGRAM_ID, potionMint, potionToken, walletKey, [], 1),
        let mint_to_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::MintTo {
                mint: potion_mint.to_account_info(),
                to: potion.to_account_info(),
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::mint_to(mint_to_ctx, 1)
            .expect("Mint To failed.");

        // potion.fast_react_price = 5;
        // potion.cooldown_days = 7; // days, could be parameterized
        let timestamp = Clock::get()?.unix_timestamp as u64;
        potion.created_timestamp = timestamp;

        // TODO: do rest

        Ok(())
    }

    pub fn react(ctx: Context<React>) -> ProgramResult {

        let potion = &mut ctx.accounts.potion;

        let day_to_seconds = 60 * 60 * 24;
        let cooldown_days = 7;
        let cooldown_expiration = potion.created_timestamp + (cooldown_days * day_to_seconds);
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
    pub token_user_account: AccountInfo<'info>,  // User's $BAPE (or generic token) account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub potion_mint: AccountInfo<'info>, // mint for potions
    // pub potion_user_account: AccountInfo<'info>, // this is a user account for the potion token. (this will be input as empty, and initialized on the backend)
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: AccountInfo<'info>, // this just anchor.web3.SYSVAR_RENT_PUBKEY from frontend
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
    pub token_user_account: Signer<'info>,  // User's $BAPE (or generic token) account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
}

#[account]
pub struct Potion {
    pub authority: Pubkey,
    // pub fast_react_price: u64,
    // pub cooldown_days: u64,
    pub created_timestamp: u64
}

#[error]
pub enum ErrorCode {
    #[msg("This potion has already been created.")]
    AlreadyCreated,
    #[msg("This potion has not reached its cooldown period.")]
    CooldownNotReached,
}
