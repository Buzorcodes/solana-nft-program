use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token::{TokenAccount as SplTokenAccount, Token, Mint as SplMint, MintTo, mint_to},
        associated_token::AssociatedToken
    },
    mpl_token_metadata::{
        ID as METADATA_PROGRAM_ID,
        instruction::{create_master_edition_v3, create_metadata_accounts_v3},
    },
    solana_program::program::invoke
};

// Rename the program ID.
declare_id!("3ZpCZ7JPrnZ6uX3kD4qLcVdy8sQx9LQ6UixbJkf6WpKn");

// Define the main program module.
#[program]
pub mod my_unique_nft_program {
    use super::*;

    // Define the function to create a unique NFT.
    pub fn create_unique_nft(context: Context<CreateUniqueNFT>, name: String, symbol: String, uri: String) -> Result<()> {

        // Define the creator information.
        let creators = vec![
            mpl_token_metadata::state::Creator {
                address: context.accounts.creator.key(),
                verified: false,
                share: 100,
            }
        ];

        // Create metadata accounts for the unique NFT.
        let create_metadata_ix = create_metadata_accounts_v3(
            context.accounts.metadata_program.key(),
            context.accounts.metadata_account.key(),
            context.accounts.unique_nft_mint.key(),
            context.accounts.creator.key(),
            context.accounts.creator.key(),
            context.accounts.creator.key(),
            name,
            symbol,
            uri,
            Some(creators),
            0,
            false,
            false,
            None,
            None,
            None
        );

        // Invoke the metadata creation instruction.
        invoke(
            &create_metadata_ix,
            &[
                context.accounts.metadata_program.to_account_info(),
                context.accounts.metadata_account.to_account_info(),
                context.accounts.unique_nft_mint.to_account_info(),
                context.accounts.creator.to_account_info()
            ]
        )?;

        // Mint one unique NFT.
        mint_to(context.accounts.mint_context(), 1)?;

        // Create a master edition for the unique NFT.
        let create_master_edition_ix = create_master_edition_v3(
            context.accounts.metadata_program.key(),
            context.accounts.master_edition.key(),
            context.accounts.unique_nft_mint.key(),
            context.accounts.creator.key(),
            context.accounts.creator.key(),
            context.accounts.metadata_account.key(),
            context.accounts.creator.key(),
            Some(1)
        );

        // Invoke the master edition creation instruction.
        invoke(
            &create_master_edition_ix,
            &[
                context.accounts.metadata_program.to_account_info(),
                context.accounts.master_edition.to_account_info(),
                context.accounts.unique_nft_mint.to_account_info(),
                context.accounts.creator.to_account_info(),
                context.accounts.metadata_account.to_account_info()
            ]
        )?;

        Ok(())
    }
}

// Define the accounts required for the unique NFT creation.
#[derive(Accounts)]
pub struct CreateUniqueNFT<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        mint::decimals = 0,
        mint::authority = creator,
        mint::freeze_authority = creator
    )]
    pub unique_nft_mint: Account<'info, SplMint>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = unique_nft_mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, SplTokenAccount>,
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), unique_nft_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub metadata_account: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), unique_nft_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub master_edition: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        constraint = metadata_program.key() == METADATA_PROGRAM_ID
    )]
    pub metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

impl<'info> CreateUniqueNFT<'info> {
    pub fn mint_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.unique_nft_mint.to_account_info(),
            to: self.creator_token_account.to_account_info(),
            authority: self.creator.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}
