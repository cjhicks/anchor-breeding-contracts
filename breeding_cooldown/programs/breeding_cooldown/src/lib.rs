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
        let token_program = &ctx.accounts.token_program;
        let user = &ctx.accounts.user;
        let token_user_account = ctx.accounts.token_user_account.to_account_info();
        let token_mint = ctx.accounts.token_mint.to_account_info();
        let potion_mint = ctx.accounts.potion_mint.to_account_info();
        let rent = &ctx.accounts.rent;
        let system_program = &ctx.accounts.system_program.to_account_info();
        
        /*
        Validation
        */
        // TODO: check that user has enough $BAPE to get a potion

        // check if 7 days since last breeding
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        let nft_1_metadata = &mut ctx.accounts.nft_1_metadata;
        if nft_1_metadata.last_bred_timestamp < breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }
        let nft_2_metadata = &mut ctx.accounts.nft_2_metadata;
        if nft_2_metadata.last_bred_timestamp < breed_min_timestamp {
            return Err(ErrorCode::NftUsedTooSoon.into());
        }

        /* 
        Mint new NFT for potion
        */

        // TODO: I think anchor does this automatically with account(init, ...)
        // anchor.web3.SystemProgram.createAccount({fromPubkey: walletKey, newAccountPubkey: potionMint, space: MintLayout.span, lamports: rent, programId: TOKEN_PROGRAM_ID,}),

        // TODO: do I need all this below? Can I just use constraints in anchor? Test in devnet
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

        // //   createAssociatedTokenAccountInstruction(potionToken, walletKey, walletKey, potionMint),
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

        // //   Token.createMintToInstruction(TOKEN_PROGRAM_ID, potionMint, potionToken, walletKey, [], 1),
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

        potion.authority = authority;
        potion.created_timestamp = timestamp;

        // TODO: Create Potion
        // client: use PublicKey.findProgramAddress to create empty new address (state) for each NFT input
        // server: verify bape1BreedingState, etc is not already initialized, or at least not within 7 days
        // - if not, create!
        // - set egg cooldown to 7 days

        nft_1_metadata.last_bred_timestamp = timestamp;
        nft_2_metadata.last_bred_timestamp = timestamp;

        /*
        Burn $BAPE after minting potion
        */
        let burn_price = 350;
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

        Ok(())
    }

    pub fn react(ctx: Context<React>) -> ProgramResult {

        let potion = &mut ctx.accounts.potion;
        let timestamp = get_timestamp();
        let breed_min_timestamp = get_breed_min_timestamp(timestamp);

        if potion.created_timestamp < breed_min_timestamp {
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
        let potion = &mut ctx.accounts.potion;

        // TODO: do fast reaction (burn more $BAPE?)
        // TODO: check that user has enough $BAPE to get a fast reaction
        let fast_burn_price = 175; // TODO: inflation?

        // TODO: mint new NFT

        Ok(())
    }
}

#[account]
pub struct Potion {
    pub authority: Pubkey,
    pub created_timestamp: u64
}

#[account]
pub struct NftMetadata {
    pub authority: Pubkey,
    pub last_bred_timestamp: u64
}

#[derive(Accounts)]
pub struct CreatePotion<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 20)]
    pub potion: Account<'info, Potion>,
    #[account(mut)]
    pub token_user_account: AccountInfo<'info>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub potion_mint: AccountInfo<'info>, // mint for potions
    pub nft_1: AccountInfo<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 8)]  // TODO: verify seeds, or create here
    pub nft_1_metadata: Account<'info, NftMetadata>,
    pub nft_2: AccountInfo<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 8)]  // TODO: verify seeds, or create here
    pub nft_2_metadata: Account<'info, NftMetadata>,

    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
    pub system_program: Program<'info, System>, // this is just anchor.web3.SystemProgram.programId from frontend
    pub rent: AccountInfo<'info>, // this just anchor.web3.SYSVAR_RENT_PUBKEY from frontend
}

#[derive(Accounts)]
pub struct React<'info> {
    #[account(mut, has_one = authority)]
    pub potion: Account<'info, Potion>,
    pub authority: Signer<'info>,
    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FastReact<'info> {
    #[account(mut, has_one = authority)]
    pub potion: Account<'info, Potion>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub token_user_account: Signer<'info>,  // User's $BAPE account, this token type should match mint account
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,  // $BAPE mint, generic enough for any token though
    pub baby_mint: AccountInfo<'info>, // mint for baby
    pub baby_nft: AccountInfo<'info>,

    pub token_program: AccountInfo<'info>,  // this is the SPL Token Program which is owner of all token mints
}

#[error]
pub enum ErrorCode {
    #[msg("This NFT has been used for breeding in the last 7 days.")]
    NftUsedTooSoon,
    #[msg("This potion has not reached its cooldown period.")]
    CooldownNotReached,
}
