pub fn create_nft(
    ctx: Context<CreateNFT>,
    name: String,
    symbol: String,
    uri: String,
    image_cid: String,
) -> Result<()> {
    // Create mint account
    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    token::mint_to(cpi_context, 1)?;

    // Create metadata account
    let accounts = mpl_instruction::CreateMetadataAccountsV3 {
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        mint_authority: ctx.accounts.payer.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        update_authority: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let creator = vec![Creator {
        address: *ctx.accounts.payer.key,
        verified: false,
        share: 100,
    }];

    let data_v2 = DataV2 {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(creator),
        collection: None,
        uses: None,
    };

    let instruction = mpl_instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        accounts.metadata.key(),
        accounts.mint.key(),
        accounts.mint_authority.key(),
        accounts.payer.key(),
        accounts.update_authority.key(),
        data_v2.name,
        data_v2.symbol,
        data_v2.uri,
        data_v2.creators,
        data_v2.seller_fee_basis_points,
        true,
        true,
        None,
        None,
        None,
    );

    anchor_lang::solana_program::program::invoke(
        &instruction,
        &[
            accounts.metadata,
            accounts.mint,
            accounts.mint_authority,
            accounts.payer,
            accounts.update_authority,
            accounts.system_program,
            accounts.rent,
        ],
    )?;

    // Store additional data on-chain
    let nft_data = &mut ctx.accounts.nft_data;
    nft_data.name = name;
    nft_data.symbol = symbol;
    nft_data.uri = uri;
    nft_data.image_cid = image_cid;
    nft_data.mint = ctx.accounts.mint.key();
    nft_data.owner = ctx.accounts.payer.key();

    emit!(NFTCreated {
        mint: ctx.accounts.mint.key(),
        name: nft_data.name.clone(),
        symbol: nft_data.symbol.clone(),
        uri: nft_data.uri.clone(),
        image_cid: nft_data.image_cid.clone(),
    });

    Ok(())
}

pub fn update_nft_data(
    ctx: Context<UpdateNFTData>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
    image_cid: Option<String>,
) -> Result<()> {
    let nft_data = &mut ctx.accounts.nft_data;

    if let Some(new_name) = name {
        nft_data.name = new_name;
    }
    if let Some(new_symbol) = symbol {
        nft_data.symbol = new_symbol;
    }
    if let Some(new_uri) = uri {
        nft_data.uri = new_uri;
    }
    if let Some(new_image_cid) = image_cid {
        nft_data.image_cid = new_image_cid;
    }

    emit!(NFTUpdated {
        mint: nft_data.mint,
        name: nft_data.name.clone(),
        symbol: nft_data.symbol.clone(),
        uri: nft_data.uri.clone(),
        image_cid: nft_data.image_cid.clone(),
    });

    Ok(())
}

#[event]
pub struct NFTCreated {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub image_cid: String,
}

#[event]
pub struct NFTUpdated {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub image_cid: String,
}
