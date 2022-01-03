use anchor_lang::prelude::*;
use anchor_spl;
use spl_token;
use solana_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

fn get_timestamp() -> u64 {
    return Clock::get().unwrap().unix_timestamp as u64;
}

fn get_breed_min_timestamp(timestamp: u64) -> u64 {
    let seven_days_in_seconds = 7 * 24 * 60 * 60;
    return timestamp - seven_days_in_seconds;
}

#[program]
pub mod breeding_cooldown {
    use super::*;

    /*
    This function is equivalent to breeding an egg: https://explorer.solana.com/tx/g5fg51XveddE1MyU3GsEUpU6e3vUz1BhWNBvye6hBziDZbKsBv4H1UjLEKr1rjLFtABt6YNM6TBBoMzDxtQ5td5
    */
    pub fn create_potion(ctx: Context<CreatePotion>, authority: Pubkey) -> ProgramResult {
        let potion = &mut ctx.accounts.potion;
        // let program_id = &ctx.program_id;
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_mint = ctx.accounts.token_mint.to_account_info();
        // let rent = &ctx.accounts.rent;
        // let system_program = &ctx.accounts.system_program.to_account_info();
        
        /*
        Validation
        */
        // check if we have enough $BAPE before continuing
        let token_user_account = &ctx.accounts.token_user_account;
        let burn_price = 350;
        if token_user_account.amount < burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        let nft_1_metadata = &mut ctx.accounts.nft_1_metadata;
        if nft_1_metadata.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }
        let nft_2_metadata = &mut ctx.accounts.nft_2_metadata;
        if nft_2_metadata.last_bred_timestamp > breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        /* 
        Mint new NFT for potion
        */
        // TODO: create / init potion user account
        // TODO: mint 1 NFT to user account
        // TODO: freeze account!
        // let potion_mint = ctx.accounts.potion_mint.to_account_info();

        // TODO: I think anchor does this automatically with account(init, ...)
        // anchor.web3.SystemProgram.createAccount({fromPubkey: walletKey, newAccountPubkey: potionMint, space: MintLayout.span, lamports: rent, programId: TOKEN_PROGRAM_ID,}),

        // Token.createInitMintInstruction(TOKEN_PROGRAM_ID, potion_mint, 0, walletKey, walletKey)
        // let init_mint_ctx = CpiContext::new(
        //     token_program.clone(),
        //     anchor_spl::token::InitializeMint {
        //         mint: potion_mint.to_account_info(),
        //         rent: rent.to_account_info()
        //     }
        // );
        // anchor_spl::token::initialize_mint(init_mint_ctx, 0, &user.key(), Some(&user.key()))
        //     .expect("Init Mint failed.");

        // createAssociatedTokenAccountInstruction(potionToken, walletKey, walletKey, potionMint),
        // let create_associated_token_ctx = CpiContext::new(
        //     token_program.clone(),
        //     anchor_spl::associated_token::Create {
        //         payer: user.to_account_info(),
        //         associated_token: potion.to_account_info(),
        //         authority: user.to_account_info(),
        //         mint: potion_mint.to_account_info(),
        //         system_program: system_program.to_account_info(),
        //         token_program: token_program.to_account_info(),
        //         rent: rent.clone(),
        //     }
        // );
        // anchor_spl::associated_token::create(create_associated_token_ctx)
        //     .expect("Create Associated Token failed.");

        //   Token.createMintToInstruction(TOKEN_PROGRAM_ID, potionMint, potionToken, walletKey, [], 1),
        // let mint_to_ctx = CpiContext::new(
        //     token_program.clone(),
        //     anchor_spl::token::MintTo {
        //         mint: potion_mint.to_account_info(),
        //         to: potion.to_account_info(),
        //         authority: user.to_account_info(),
        //     }
        // );
        // anchor_spl::token::mint_to(mint_to_ctx, 1)
        //     .expect("Mint To failed.");

        // TODO: if this works ^ need to freeze account!

        let nft_1_key = *ctx.accounts.nft_1.key;
        let nft_2_key = *ctx.accounts.nft_2.key;
        potion.authority = authority;
        potion.nft1 = nft_1_key;
        potion.nft2 = nft_2_key;
        potion.created_timestamp = timestamp;

        nft_1_metadata.authority = authority;
        nft_1_metadata.nft = nft_1_key;
        nft_1_metadata.last_bred_timestamp = timestamp;
        nft_2_metadata.authority = authority;
        nft_2_metadata.nft = nft_2_key;
        nft_2_metadata.last_bred_timestamp = timestamp;

        /*
        Burn $BAPE after minting potion
        */
        let burn_ctx = CpiContext::new(
            token_program.clone(),
            anchor_spl::token::Burn {
                to: token_user_account.to_account_info(),
                mint: token_mint,
                authority: user.to_account_info(),
            }
        );
        anchor_spl::token::burn(burn_ctx, burn_price)
            .expect("burn failed.");

        Ok(())
    }

    pub fn react(ctx: Context<React>) -> ProgramResult {

        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        /*
        Validations (function)
        1. User is Authority on Potion
        2. NFT metadata matches Potion and Authority
        3. Created Timestamp > 7 days
        4. Verify Mint on egg is legit
        */
        let user_key = *ctx.accounts.user.key;
        let potion = &mut ctx.accounts.potion;
        let nft_1_metadata = &ctx.accounts.nft_1_metadata;
        let nft_2_metadata = &ctx.accounts.nft_2_metadata;

        if potion.authority != user_key || nft_1_metadata.authority != user_key || nft_2_metadata.authority != user_key {
            return Err(ErrorCode::Unauthorized.into());
        }

        if !((potion.nft1 == nft_1_metadata.nft && potion.nft2 == nft_2_metadata.nft) ||
            (potion.nft1 == nft_2_metadata.nft && potion.nft2 == nft_1_metadata.nft)) {
            return Err(ErrorCode::Mismatch.into());
        }

        if potion.created_timestamp > breed_min_timestamp {
            return Err(ErrorCode::CooldownNotReached.into());
        }

        // TODO: mint new NFT (master edition)
        // TODO: make this a reusable function

        // TODO: Create baby
        // client: use PublicKey.findProgramAddress to create empty (existing) address (state) for each NFT input
        // server: verify egg over 7 days - no parent necessary?
        // - or, should we require pass it in to verify its still held?

        // FIrst Instructions
        // 1 - Create Account (walletKey, new=babyMint)
        // 2 - Init Mint (walletKey, mint: babyMint)
        // 3 - Associated Token Create (account=babyToken, mint=babyMint, walletKey)
        // 4 - MintTo (token=babyMint, account=babyToken, mint=BabyMint, authority=walletKey)

        // Unknown Program Instructions

        Ok(())
    }

    pub fn fast_react(ctx: Context<FastReact>) -> ProgramResult {
        /*
        Validations (function)
        1. User is Authority on Potion
        2. NFT metadata matches Potion and Authority
        3. User has enough $BAPE
        4. Verify Mint on egg is legit
        */
        let user_key = *ctx.accounts.user.key;
        let potion = &mut ctx.accounts.potion;
        let nft_1_metadata = &ctx.accounts.nft_1_metadata;
        let nft_2_metadata = &ctx.accounts.nft_2_metadata;

        if potion.authority != user_key || nft_1_metadata.authority != user_key || nft_2_metadata.authority != user_key {
            return Err(ErrorCode::Unauthorized.into());
        }

        if !((potion.nft1 == nft_1_metadata.nft && potion.nft2 == nft_2_metadata.nft) ||
            (potion.nft1 == nft_2_metadata.nft && potion.nft2 == nft_1_metadata.nft)) {
            return Err(ErrorCode::Mismatch.into());
        }
        // TODO: do fast reaction (burn more $BAPE?)
        let token_user_account = &ctx.accounts.token_user_account;
        let fast_burn_price = 175;
        if token_user_account.amount < fast_burn_price {
            return Err(ErrorCode::InsufficientFunds.into())
        }

        // TODO: mint new NFT

        // TODO: after mint successful, burn 175 $BAPE

        Ok(())
    }
}

#[account]
pub struct Potion {
    pub authority: Pubkey,
    pub nft1: Pubkey,
    pub nft2: Pubkey,
    pub created_timestamp: u64
}

#[account]
pub struct NftMetadata {
    pub authority: Pubkey,
    pub nft: Pubkey,
    pub last_bred_timestamp: u64
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 120)]
    pub potion: Account<'info, Potion>,
    // TODO: owner = user?
    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though
    // #[account(mut, owner = user)]
    // #[account(mut)]
    // pub potion_mint: Account<'info, anchor_spl::token::Mint>, // mint for potions
    // TODO: owner = user?
    // #[account(init, payer = user, space = 8 + 40)]
    // pub potion_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    // TODO: owner is user
    // #[account(owner = *user.key)]
    pub nft_1: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump, payer = user, space = 8 + 80)]
    pub nft_1_metadata: Account<'info, NftMetadata>,
    // #[account(owner = *user.key)]  -> Might need something else here
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump, payer = user, space = 8 + 80)]    // pub nft_2_metadata: Account<'info, NftMetadata>,
    pub nft_2_metadata: Account<'info, NftMetadata>,
    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: AccountInfo<'info>, // this just anchor.web3.SYSVAR_RENT_PUBKEY from frontend
}

#[derive(Accounts)]
pub struct React<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, Potion>,

    pub nft_1: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump)] 
    pub nft_1_metadata: Account<'info, NftMetadata>,
    pub nft_2: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump)] 
    pub nft_2_metadata: Account<'info, NftMetadata>,

    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FastReact<'info> {
    pub user: Signer<'info>,
    #[account(mut, owner = *user.key)]
    pub potion: Account<'info, Potion>,

    pub nft_1: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_1.key.as_ref()], bump)] 
    pub nft_1_metadata: Account<'info, NftMetadata>,
    pub nft_2: AccountInfo<'info>,
    #[account(seeds = [b"bapeBreeding".as_ref(), nft_2.key.as_ref()], bump)] 
    pub nft_2_metadata: Account<'info, NftMetadata>,

    #[account(mut)]
    pub token_user_account: Account<'info, anchor_spl::token::TokenAccount>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,  // $BAPE mint, generic enough for any token though

    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,

    #[account(executable, "token_program.key == &anchor_spl::token::ID")]
    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
}

#[error]
pub enum ErrorCode {
    #[msg("This NFT has been used for breeding in the last 7 days.")]
    NftUsedTooSoon,
    #[msg("This potion has not reached its cooldown period.")]
    CooldownNotReached,
    #[msg("User has insufficient funds to complete the transaction.")]
    InsufficientFunds,
    #[msg("User is not authorized to complete the transaction.")]
    Unauthorized,
    #[msg("NFT's do not match token provided.")]
    Mismatch,
}
