import { AnchorProvider } from '@coral-xyz/anchor'
import { Connection, type PublicKey } from '@solana/web3.js'
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom'
import { derived, writable } from 'svelte/store'


type WalletState = {
	connected: boolean
	address: string | null
	publicKey: PublicKey | null
}

const adapter = new PhantomWalletAdapter()
const state = writable<WalletState>({ connected: false, address: null, publicKey: null })

function syncFromAdapter() {
	const publicKey = adapter.publicKey ?? null
	state.set({
		connected: Boolean(adapter.connected && publicKey),
		address: publicKey?.toBase58() ?? null,
		publicKey
	})
}

adapter.on('connect', syncFromAdapter)
adapter.on('disconnect', syncFromAdapter)

export const connection = new Connection(import.meta.env.VITE_RPC_URL ?? 'http://127.0.0.1:8899', 'confirmed')

function createWallet() {
	return {
		subscribe: state.subscribe,
		async connect() {
			await adapter.connect()
			syncFromAdapter()
		},
		async disconnect() {
			await adapter.disconnect()
			syncFromAdapter()
		}
	}
}

export const wallet = createWallet()

export const displayAddress = derived(state, ($w) => {
	if (!$w.connected || !$w.address) return 'Connect Wallet'
	const a = $w.address
	return `${a.slice(0, 4)}...${a.slice(-4)}`
})

export function getProvider(): AnchorProvider {
	if (!adapter.publicKey) {
		throw new Error('Wallet not connected')
	}
	if (!adapter.signTransaction || !adapter.signAllTransactions) {
		throw new Error('Connected wallet does not support transaction signing')
	}

	const walletForAnchor = {
		publicKey: adapter.publicKey,
		signTransaction: adapter.signTransaction.bind(adapter),
		signAllTransactions: adapter.signAllTransactions.bind(adapter)
	}

	return new AnchorProvider(connection, walletForAnchor as never, {
	    preflightCommitment: 'processed',
	    commitment: 'confirmed',
	    skipPreflight: false
	})
}

// Temporary compatibility shims until profile page is fully migrated to on-chain data.
export function getUsername(_address: string): string {
	return ''
}

export function setUsername(_address: string, _name: string) {
	return
}
