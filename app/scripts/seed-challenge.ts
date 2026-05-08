import { readFileSync } from 'fs'
import { homedir } from 'os'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { Connection, Keypair, PublicKey } from '@solana/web3.js'
import { AnchorProvider, Program, BN } from '@coral-xyz/anchor'
import type { Idl } from '@coral-xyz/anchor'


const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const IDL_PATH = join(__dirname, '..', '..', 'target', 'idl', 'neofit.json')
const KEYPAIR_PATH = join(homedir(), '.config', 'solana', 'id.json')
const PROGRAM_ID = new PublicKey('BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5')
const RPC_URL = 'http://127.0.0.1:8899'


async function main() {
  const keypairData = JSON.parse(readFileSync(KEYPAIR_PATH, 'utf-8'))
  const authority = Keypair.fromSecretKey(Uint8Array.from(keypairData))

  const connection = new Connection(RPC_URL, 'confirmed')
  const wallet = {
    publicKey: authority.publicKey,
    signTransaction: async (tx: any) => { tx.sign(authority); return tx },
    signAllTransactions: async (txs: any[]) => { txs.forEach(tx => tx.sign(authority)); return txs }
  }
  const provider = new AnchorProvider(connection, wallet as any, { commitment: 'confirmed' })

  const idlJson = JSON.parse(readFileSync(IDL_PATH, 'utf-8'))
  // Merge type definitions into accounts for Anchor 0.32.x compatibility
  if (idlJson.accounts && idlJson.types) {
    idlJson.accounts = idlJson.accounts.map((acc: any) => {
      if (acc.type) return acc
      const typeDef = idlJson.types.find((t: any) => t.name.toLowerCase() === acc.name.toLowerCase())
      return typeDef ? { ...acc, type: typeDef.type } : acc
    })
  }

  const ProgramCtor = Program as unknown as new (...args: any[]) => any
  const program = new ProgramCtor(idlJson as Idl, provider)

  const nonce = new BN(1)
  const title = 'Weekend Warrior'
  const requirements = [
    { exerciseId: 0, repTarget: 100 },  // 100 squats
    { exerciseId: 1, repTarget: 50 },   // 50 pushups
  ]
  const entryFee = new BN(100_000_000) // 0.1 SOL in lamports
  const deadline = new BN(Math.floor(Date.now() / 1000) + 48 * 60 * 60) // 48 hours from now

  // Derive challenge PDA
  const nonceBuf = Buffer.alloc(8)
  nonceBuf.writeBigUInt64LE(BigInt(1))
  const [challengePda] = PublicKey.findProgramAddressSync(
    [Buffer.from('challenge'), authority.publicKey.toBuffer(), nonceBuf],
    PROGRAM_ID
  )

  console.log('Creating challenge...')
  console.log('  Authority:', authority.publicKey.toBase58())
  console.log('  Challenge PDA:', challengePda.toBase58())

  const tx = await program.methods
    .createChallenge(title, requirements, entryFee, deadline, nonce)
    .accounts({
      challenge: challengePda,
      authority: authority.publicKey,
      systemProgram: new PublicKey('11111111111111111111111111111111')
    })
    .signers([authority])
    .rpc()

  console.log('  TX Signature:', tx)
  console.log('\nDone! Verify with:')
  console.log(`  solana account ${challengePda.toBase58()} --url localhost`)
}

main().catch(console.error)

