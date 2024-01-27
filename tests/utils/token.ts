import * as anchor from '@project-serum/anchor';
import mintAuth from '../keypairs/mint-authority.json';

import {
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';

const generateCreateTokenIx = async (
  provider: anchor.Provider,
  mint: anchor.web3.PublicKey,
  recipient: anchor.web3.PublicKey,
  mintAuth: anchor.web3.Keypair
) => {
  const tokenAddress = await getAssociatedTokenAddress(mint, recipient);
  return {
    tokenAddress,
    createATAIx: (await provider.connection.getAccountInfo(tokenAddress))
      ? null
      : createAssociatedTokenAccountInstruction(
          mintAuth.publicKey,
          tokenAddress,
          recipient,
          mint
        ),
  };
};

export const createAssocAccount = async (
  provider: anchor.Provider,
  recipient: anchor.web3.PublicKey
) => {
  const { web3 } = anchor;
  const dtrkMint = new web3.PublicKey(
    'DTRK1XRNaL6CxfFWwVLZMxyhiCZnwdP32CgzVDXWy5Td'
  );

  const mintAuthKeypair = web3.Keypair.fromSecretKey(Uint8Array.from(mintAuth));

  const { tokenAddress, createATAIx } = await generateCreateTokenIx(
    provider,
    dtrkMint,
    recipient,
    mintAuthKeypair
  );

  console.log('Token address', tokenAddress.toString());

  const transaction = new web3.Transaction().add(createATAIx);
  await provider.connection.confirmTransaction(
    await provider.connection.sendTransaction(transaction, [mintAuthKeypair]),
    'singleGossip'
  );
};
