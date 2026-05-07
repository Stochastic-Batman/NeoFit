<script lang="ts">
	import { wallet } from '$lib/wallet'
	import { fetchUserProfile, initializeUser, updateUsername } from '$lib/program'
	import { PublicKey } from '@solana/web3.js'

	let username = $state('')
	let editingName = $state(false)
	let draft = $state('')
	let totalReps = $state(0)
	let streakDays = $state(0)
	let profileExists = $state<boolean | null>(null) // null = loading
	let loading = $state(false)
	let error = $state('')

	$effect(() => {
		if ($wallet.connected && $wallet.publicKey) {
			loadProfile($wallet.publicKey)
		} else {
			profileExists = null
			username = ''
			totalReps = 0
			streakDays = 0
		}
	})

	async function loadProfile(pubkey: PublicKey) {
		try {
			error = ''
			const data = await fetchUserProfile(pubkey)
			if (data) {
				profileExists = true
				username = data.username ?? ''
				totalReps = Number(data.totalReps ?? 0)
				streakDays = Number(data.streakDays ?? 0)
			} else {
				profileExists = false
			}
		} catch (e: any) {
			error = e.message
			profileExists = false
		}
	}

	async function handleCreateProfile() {
		loading = true
		error = ''
		try {
			await initializeUser()
			if ($wallet.publicKey) await loadProfile($wallet.publicKey)
		} catch (e: any) {
			error = e.message
		} finally {
			loading = false
		}
	}

	async function saveName() {
		if (!draft.trim()) { editingName = false; return }
		loading = true
		error = ''
		try {
			await updateUsername(draft.trim())
			username = draft.trim()
			editingName = false
		} catch (e: any) {
			error = e.message
		} finally {
			loading = false
		}
	}
</script>

{#if !$wallet.connected}
	<div class="flex flex-col items-center justify-center min-h-[50vh] text-center gap-6">
		<h2 class="font-orbitron text-2xl uppercase tracking-wider">Wallet Required</h2>
		<p class="text-white/40">Connect your wallet to view your profile.</p>
	</div>
{:else if profileExists === null}
	<div class="flex items-center justify-center min-h-[50vh]">
		<p class="font-mono text-white/40 animate-pulse">Loading profile...</p>
	</div>
{:else if profileExists === false}
	<div class="flex flex-col items-center justify-center min-h-[50vh] text-center gap-6">
		<h2 class="font-orbitron text-2xl uppercase tracking-wider">No Profile Found</h2>
		<p class="text-white/40">Create your on-chain profile to start tracking reps.</p>
		{#if error}
			<p class="text-red-400 text-xs font-mono">{error}</p>
		{/if}
		<button
			onclick={handleCreateProfile}
			disabled={loading}
			class="font-orbitron text-sm uppercase tracking-widest px-8 py-3 border border-[#95389E] text-[#95389E] hover:bg-[#95389E] hover:text-white transition-all duration-300 disabled:opacity-50">
			{loading ? 'Creating...' : 'Create Profile'}
		</button>
	</div>
{:else}
	<div class="max-w-4xl mx-auto">
		<header class="mb-12 border-b border-white/10 pb-8 flex flex-col md:flex-row md:justify-between md:items-end gap-4">
			<div>
				<h1 class="font-orbitron text-4xl tracking-wider uppercase mb-2 text-[#F7F7F7]">Identity Profile</h1>
				<p class="font-mono text-sm text-[#43D8C9]">
					{$wallet.address?.slice(0, 6)}…{$wallet.address?.slice(-6)}
				</p>
			</div>

			<!-- Username block -->
			<div class="text-left md:text-right">
				{#if editingName}
					<div class="flex gap-2 items-center">
						<input
							bind:value={draft}
							maxlength={22}
							placeholder="your_handle"
							class="bg-transparent border border-[#43D8C9] text-white font-mono text-sm px-3 py-1 focus:outline-none w-40"
						/>
						<button
							onclick={saveName}
							disabled={loading}
							class="font-orbitron text-[10px] uppercase tracking-widest border border-[#43D8C9] text-[#43D8C9] px-3 py-1 hover:bg-[#43D8C9]/10 transition-colors disabled:opacity-50">
							{loading ? '...' : 'Save'}
						</button>
						<button
							onclick={() => editingName = false}
							class="font-orbitron text-[10px] uppercase tracking-widest text-white/30 hover:text-white/60 transition-colors">
							✕
						</button>
					</div>
				{:else}
					<button
						onclick={() => { draft = username; editingName = true }}
						class="group text-left md:text-right">
						{#if username}
							<p class="font-orbitron text-[#95389E] font-bold text-xl">{username}</p>
							<p class="text-[10px] uppercase tracking-widest text-white/20 group-hover:text-white/40 transition-colors">Edit handle</p>
						{:else}
							<p class="font-orbitron text-white/20 text-sm hover:text-white/50 transition-colors">+ Set username</p>
						{/if}
					</button>
				{/if}
			</div>
		</header>

		{#if error}
			<div class="mb-6 border border-red-500/20 bg-red-500/5 px-4 py-2">
				<p class="text-red-400 text-xs font-mono">{error}</p>
			</div>
		{/if}

		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden">
				<div class="absolute bottom-0 left-0 w-full h-1 bg-[#95389E]"></div>
				<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Lifetime Reps</p>
				<p class="font-orbitron text-5xl font-black">{totalReps}</p>
			</div>

			<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden">
				<div class="absolute bottom-0 left-0 w-full h-1 bg-[#43D8C9]"></div>
				<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Active Streak</p>
				<p class="font-orbitron text-5xl font-black flex items-baseline gap-2">
					{streakDays} <span class="text-sm font-sans font-normal text-[#43D8C9]">DAYS</span>
				</p>
			</div>
		</div>

		<div class="mt-16 p-8 border border-white/10 bg-black/20">
			<h2 class="font-orbitron text-xl mb-6 text-white/80">Recent Ledger Activity</h2>
			<p class="font-mono text-sm text-white/30">Transaction history coming soon.</p>
		</div>
	</div>
{/if}
