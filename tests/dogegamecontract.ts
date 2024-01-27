import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import {
  ASSOCIATED_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from '@project-serum/anchor/dist/cjs/utils/token';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { Dogegamecontract } from '../target/types/dogegamecontract';
import { Metadata } from '@metaplex-foundation/mpl-token-metadata';

import trackHolderKeypair from './keypairs/track-holder.json';
import stateAuthorityKeypair from './keypairs/state-authority.json';
import dogeHolderKeypair from './keypairs/doge-holder.json';

import { createAssocAccount } from './utils/token';
import { assert, expect } from 'chai';

async function sleep(timeInMs: number) {
  return new Promise((res) => setTimeout(res, timeInMs));
}

describe('dogegamecontract', () => {
  const {
    web3: { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY },
  } = anchor;

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const { provider, methods, programId, account } = anchor.workspace
    .Dogegamecontract as Program<Dogegamecontract>;

  const trackHolder = Keypair.fromSecretKey(
    Uint8Array.from(trackHolderKeypair)
  );

  const stateAuthority = Keypair.fromSecretKey(
    Uint8Array.from(stateAuthorityKeypair)
  );

  const dogeHolder = Keypair.fromSecretKey(Uint8Array.from(dogeHolderKeypair));

  const dtrkMint = new PublicKey(
    'DTRK1XRNaL6CxfFWwVLZMxyhiCZnwdP32CgzVDXWy5Td'
  );

  // it('Creates Token account', async () => {
  // await createAssocAccount(provider, trackHolder.publicKey);
  // await createAssocAccount(provider, stateAuthority.publicKey);
  // await createAssocAccount(provider, dogeHolder.publicKey);
  // });

  it('Creates a new Lobby', async () => {
    console.log('Program ID:', programId.toString());
    console.log('Track holder:', trackHolder.publicKey.toString());
    console.log('State Authority:', stateAuthority.publicKey.toString(), '\n');

    const trackMint = new PublicKey(
      'GSkukc1RxwFUo3LAFEsVfEjgEUWLrKorwCjZcdSeGZTu'
    );

    const [lobbyAccount] = await PublicKey.findProgramAddress(
      [
        Buffer.from('lobby'),
        trackHolder.publicKey.toBuffer(),
        trackMint.toBuffer(),
      ],
      programId
    );

    const lobbyMetadata = {
      name: 'ChillThrill',
      location: 'Solana Beach',
      entryFee: new anchor.BN(120),
      minClass: 3,
      totalLaps: 4,
      trackType: { sand: {} },
    };

    const lobbyDtrkToken = await getAssociatedTokenAddress(
      dtrkMint,
      lobbyAccount,
      true
    );
    const lobbyTrackToken = await getAssociatedTokenAddress(
      trackMint,
      lobbyAccount,
      true
    );
    const trackHolderDtrk = await getAssociatedTokenAddress(
      dtrkMint,
      trackHolder.publicKey
    );
    const trackHolderToken = await getAssociatedTokenAddress(
      trackMint,
      trackHolder.publicKey
    );

    const trackMetadata = await Metadata.getPDA(trackMint);
    await methods
      .createLobby(lobbyMetadata)
      .accounts({
        trackHolder: trackHolder.publicKey,
        stateAuthority: stateAuthority.publicKey,
        lobbyAccount,
        // track keys
        dtrkMint,
        trackMint,
        trackMetadata,
        lobbyDtrkToken,
        lobbyTrackToken,
        trackHolderDtrk,
        trackHolderToken,
        // programs
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        // sysvar
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([trackHolder, stateAuthority])
      .rpc({ commitment: 'singleGossip', skipPreflight: true });

    const unlockTime = Math.floor(Date.now() / 1000) + 10;

    const lobbyState = await account.lobbyState.fetch(lobbyAccount);
    // keys
    expect(lobbyState.stateAuthority.toString()).eql(
      stateAuthority.publicKey.toString()
    );
    expect(lobbyState.trackKeys.lobbyDtrkToken.toString()).eql(
      lobbyDtrkToken.toString()
    );
    expect(lobbyState.trackKeys.lobbyTrackToken.toString()).eql(
      lobbyTrackToken.toString()
    );
    expect(lobbyState.trackKeys.trackHolderDtrk.toString()).eql(
      trackHolderDtrk.toString()
    );
    expect(lobbyState.trackKeys.trackHolderToken.toString()).eql(
      trackHolderToken.toString()
    );
    expect(lobbyState.trackKeys.trackMetadata.toString()).eql(
      trackMetadata.toString()
    );
    expect(lobbyState.trackKeys.trackMint.toString()).eql(trackMint.toString());
    // expect(lobbyState.unlockTime.toNumber()).approximately(unlockTime, 1);
    // track metadata
    expect(lobbyState.lobbyData.entryFee.toNumber()).eql(
      lobbyMetadata.entryFee.toNumber()
    );
    expect(lobbyState.lobbyData.name).eql(lobbyMetadata.name);
    expect(lobbyState.lobbyData.location).eql(lobbyMetadata.location);
    expect(lobbyState.lobbyData.minClass).eql(lobbyMetadata.minClass);
    expect(lobbyState.lobbyData.totalLaps).eql(lobbyMetadata.totalLaps);
    expect(JSON.stringify(lobbyState.lobbyData.trackType)).to.equal(
      '{"sand":{}}'
    );
  });

  it('Updates the lobby', async () => {
    const trackMint = new PublicKey(
      'GSkukc1RxwFUo3LAFEsVfEjgEUWLrKorwCjZcdSeGZTu'
    );

    console.log('Sleeping for 10s');
    await sleep(10 * 1000);
    const [lobbyAccount] = await PublicKey.findProgramAddress(
      [
        Buffer.from('lobby'),
        trackHolder.publicKey.toBuffer(),
        trackMint.toBuffer(),
      ],
      programId
    );

    const lobbyDtrkToken = await getAssociatedTokenAddress(
      dtrkMint,
      lobbyAccount,
      true
    );
    const lobbyTrackToken = await getAssociatedTokenAddress(
      trackMint,
      lobbyAccount,
      true
    );

    const newLobbyMetadata = {
      name: 'ChillThrill New',
      location: 'Solana Heaven',
      entryFee: new anchor.BN(150),
      minClass: 4,
      totalLaps: 4,
      trackType: { pavement: {} },
    };

    await methods
      .updateLobbyMetadata(newLobbyMetadata)
      .accounts({
        lobbyAccount,
        lobbyDtrkToken,
        lobbyTrackToken,
        trackHolder: trackHolder.publicKey,
        trackMint,
      })
      .signers([trackHolder])
      .rpc({ commitment: 'singleGossip', skipPreflight: true });

    const lobbyState = await account.lobbyState.fetch(lobbyAccount);
    // track metadata
    expect(lobbyState.lobbyData.entryFee.toNumber()).eql(
      newLobbyMetadata.entryFee.toNumber()
    );
    expect(lobbyState.lobbyData.name).eql(newLobbyMetadata.name);
    expect(lobbyState.lobbyData.location).eql(newLobbyMetadata.location);
    expect(lobbyState.lobbyData.minClass).eql(newLobbyMetadata.minClass);
    expect(lobbyState.lobbyData.totalLaps).eql(newLobbyMetadata.totalLaps);
    expect(JSON.stringify(lobbyState.lobbyData.trackType)).to.equal(
      '{"pavement":{}}'
    );
  });

  it('Closes the lobby', async () => {
    const trackMint = new PublicKey(
      'GSkukc1RxwFUo3LAFEsVfEjgEUWLrKorwCjZcdSeGZTu'
    );

    const [lobbyAccount] = await PublicKey.findProgramAddress(
      [
        Buffer.from('lobby'),
        trackHolder.publicKey.toBuffer(),
        trackMint.toBuffer(),
      ],
      programId
    );

    const lobbyDtrkToken = await getAssociatedTokenAddress(
      dtrkMint,
      lobbyAccount,
      true
    );
    const lobbyTrackToken = await getAssociatedTokenAddress(
      trackMint,
      lobbyAccount,
      true
    );
    const trackHolderToken = await getAssociatedTokenAddress(
      trackMint,
      trackHolder.publicKey
    );

    await methods
      .closeLobby()
      .accounts({
        dtrkMint,
        trackMint,
        lobbyAccount,
        lobbyDtrkToken,
        lobbyTrackToken,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        trackHolder: trackHolder.publicKey,
        trackHolderToken,
      })
      .signers([trackHolder])
      .rpc({ skipPreflight: true, commitment: 'singleGossip' });
  });
});
