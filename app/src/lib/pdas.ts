import { PublicKey } from '@solana/web3.js'

const PROGRAM_ID = new PublicKey(
	import.meta.env.VITE_PROGRAM_ID ?? 'BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5'
)

export function userProfilePda(authority: PublicKey): [PublicKey, number] {
	return PublicKey.findProgramAddressSync([Buffer.from('user_profile'), authority.toBuffer()], PROGRAM_ID)
}

export function challengePda(authority: PublicKey, nonce: bigint | number): [PublicKey, number] {
	const nonceBuf = Buffer.alloc(8)
	nonceBuf.writeBigUInt64LE(typeof nonce === 'bigint' ? nonce : BigInt(nonce))

	return PublicKey.findProgramAddressSync(
		[Buffer.from('challenge'), authority.toBuffer(), nonceBuf],
		PROGRAM_ID
	)
}

export function enrollmentPda(challenge: PublicKey, user: PublicKey): [PublicKey, number] {
	return PublicKey.findProgramAddressSync(
		[Buffer.from('enrollment'), challenge.toBuffer(), user.toBuffer()],
		PROGRAM_ID
	)
}
