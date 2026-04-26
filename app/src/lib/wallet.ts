import { writable, derived } from 'svelte/store'

const ENV_ADDR = import.meta.env.VITE_PUBLIC_WALLET_ADDRESS as string | undefined

type WalletState = {
	connected: boolean
	address: string | null
}

function createWallet() {
	const { subscribe, set, update } = writable<WalletState>({ connected: false, address: null })

	return {
		subscribe,
		connect() {
			const address = ENV_ADDR ?? 'ENV_ADDR_NOT_SPECIFIED'
			set({ connected: true, address })
		},
		disconnect() {
			update((s) => ({ ...s, connected: false }))
		}
	}
}

export const wallet = createWallet()

// Stored as localStorage["neofit:username:<address>"] so each wallet address
// has its own independent username. When the backend lands this moves to a PDA.

function usernameKey(address: string) {
	return `neofit:username:${address}`
}

export function getUsername(address: string): string {
	return localStorage.getItem(usernameKey(address)) ?? 'default_as_irl'
}

export function setUsername(address: string, name: string) {
	if (name.trim()) localStorage.setItem(usernameKey(address), name.trim())
	else localStorage.removeItem(usernameKey(address))
}

// Derived readable: display string for the nav button
export const displayAddress = derived(wallet, ($w) => {
	if (!$w.connected || !$w.address) return 'Connect Wallet'
	const a = $w.address
	return `${a.slice(0, 4)}…${a.slice(-4)}`
})
