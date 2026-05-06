import { Program, type Idl } from '@coral-xyz/anchor'
import { PublicKey, SystemProgram } from '@solana/web3.js'
import { enrollmentPda, userProfilePda } from '$lib/pdas'
import { getProvider } from '$lib/wallet'
import idlJson from '$lib/idl/neofit.json'

const PROGRAM_ID = new PublicKey(
	import.meta.env.VITE_PROGRAM_ID ?? 'BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5'
)

function getProgram() {
	const provider = getProvider()
	const ProgramCtor = Program as unknown as new (...args: any[]) => any
	return new ProgramCtor(idlJson as Idl, PROGRAM_ID, provider)
}

function toErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return 'Unknown transaction error'
}

export async function initializeUser(): Promise<string> {
	try {
		const program = getProgram()
		const authority = program.provider.wallet.publicKey as PublicKey
		const [userProfile] = userProfilePda(authority)

		return await program.methods
			.initializeUser()
			.accounts({
				userProfile,
				authority,
				systemProgram: SystemProgram.programId
			})
			.rpc()
	} catch (error) {
		throw new Error(initializeUser failed: ${toErrorMessage(error)})
	}
}

export async function fetchUserProfile(authority: PublicKey): Promise<any | null> {
	try {
		const program = getProgram()
		const [userProfile] = userProfilePda(authority)
		return await program.account.userProfile.fetchNullable(userProfile)
	} catch (error) {
		throw new Error(fetchUserProfile failed: ${toErrorMessage(error)})
	}
}

export async function updateUsername(newUsername: string): Promise<string> {
	try {
		const program = getProgram()
		const authority = program.provider.wallet.publicKey as PublicKey
		const [userProfile] = userProfilePda(authority)

		return await program.methods
			.updateUsername(newUsername)
			.accounts({ userProfile, authority })
			.rpc()
	} catch (error) {
		throw new Error(updateUsername failed: ${toErrorMessage(error)})
	}
}

export async function logReps(
	exerciseId: number,
	count: number,
	challengeKey?: PublicKey
): Promise<string> {
	try {
		const program = getProgram()
		const authority = program.provider.wallet.publicKey as PublicKey
		const [userProfile] = userProfilePda(authority)

		const accounts: Record<string, PublicKey> = {
			userProfile,
			authority
		}

		if (challengeKey) {
			const [enrollment] = enrollmentPda(challengeKey, authority)
			accounts.challenge = challengeKey
			accounts.enrollment = enrollment
		}

		return await program.methods
			.logReps(exerciseId, count)
			.accounts(accounts)
			.rpc()
	} catch (error) {
		throw new Error(logReps failed: ${toErrorMessage(error)})
	}
}

export async function joinChallenge(challengeKey: PublicKey): Promise<string> {
	try {
		const program = getProgram()
		const authority = program.provider.wallet.publicKey as PublicKey
		const [userProfile] = userProfilePda(authority)
		const [enrollment] = enrollmentPda(challengeKey, authority)

		return await program.methods
			.joinChallenge()
			.accounts({
				enrollment,
				challenge: challengeKey,
				userProfile,
				authority,
				systemProgram: SystemProgram.programId
			})
			.rpc()
	} catch (error) {
		throw new Error(joinChallenge failed: ${toErrorMessage(error)})
	}
}

export async function claimReward(challengeKey: PublicKey): Promise<string> {
	try {
		const program = getProgram()
		const authority = program.provider.wallet.publicKey as PublicKey
		const [enrollment] = enrollmentPda(challengeKey, authority)

		return await program.methods
			.claimReward()
			.accounts({
				enrollment,
				challenge: challengeKey,
				authority,
				systemProgram: SystemProgram.programId
			})
			.rpc()
	} catch (error) {
		throw new Error(claimReward failed: ${toErrorMessage(error)})
	}
}

export async function fetchAllChallenges(): Promise<any[]> {
	try {
		const program = getProgram()
		return await program.account.challenge.all()
	} catch (error) {
		throw new Error(fetchAllChallenges failed: ${toErrorMessage(error)})
	}
}

export async function fetchEnrollment(challengeKey: PublicKey, userKey: PublicKey): Promise<any | null> {
	try {
		const program = getProgram()
		const [enrollment] = enrollmentPda(challengeKey, userKey)
		return await program.account.enrollment.fetchNullable(enrollment)
	} catch (error) {
		throw new Error(fetchEnrollment failed: ${toErrorMessage(error)})
	}
}
