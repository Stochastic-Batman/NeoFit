<script lang="ts">
	import { wallet, getUsername, setUsername } from '$lib/wallet'

	// Stored client-side in localStorage, keyed by wallet address.
	// Once the Anchor program has a UserProfile PDA, this moves on-chain.
	let username = $state('')
	let editingName = $state(false)
	let draft = $state('')

	$effect(() => {
		if ($wallet.address) username = getUsername($wallet.address)
	})

	function saveName() {
		if ($wallet.address) {
			setUsername($wallet.address, draft)
			username = draft.trim()
		}
		editingName = false
	}

	// These are hard-coded until the UserProfile PDA exists on-chain.
	// totalReps and streakDays will come from program account data.
	// solEarned will come from the reward token balance / tournament payouts.
	// rank will be derived server-side or from a bracket of totalReps.
	const userStats = {
		totalReps: 1250,
		streakDays: 12,
		solEarned: 1.45,
		rank: 'Cyber-Athlete'
	}
</script>

{#if !$wallet.connected}
	<div class="flex flex-col items-center justify-center min-h-[50vh] text-center gap-6">
		<h2 class="font-orbitron text-2xl uppercase tracking-wider">Wallet Required</h2>
		<p class="text-white/40">Connect your wallet to view your profile.</p>
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
							maxlength={24}
							placeholder="your_handle"
							class="bg-transparent border border-[#43D8C9] text-white font-mono text-sm px-3 py-1 focus:outline-none w-40"
						/>
						<button
							onclick={saveName}
							class="font-orbitron text-[10px] uppercase tracking-widest border border-[#43D8C9] text-[#43D8C9] px-3 py-1 hover:bg-[#43D8C9]/10 transition-colors">
							Save
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
				<p class="text-[10px] uppercase tracking-widest text-white/50 mt-1">{userStats.rank}</p>
			</div>
		</header>

		<!-- Stats - placeholder banner -->
		<div class="mb-6 border border-yellow-500/20 bg-yellow-500/5 px-4 py-2 flex items-center gap-3">
			<span class="text-yellow-500 text-xs">⚠</span>
			<p class="text-yellow-500/70 text-xs font-mono">
				Stats below are placeholder data. They will reflect your real on-chain PDA once the Anchor program is deployed.
			</p>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden">
				<div class="absolute bottom-0 left-0 w-full h-1 bg-[#95389E]"></div>
				<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Lifetime Reps</p>
				<p class="font-orbitron text-5xl font-black">{userStats.totalReps}</p>
			</div>

			<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden">
				<div class="absolute bottom-0 left-0 w-full h-1 bg-[#43D8C9]"></div>
				<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Active Streak</p>
				<p class="font-orbitron text-5xl font-black flex items-baseline gap-2">
					{userStats.streakDays} <span class="text-sm font-sans font-normal text-[#43D8C9]">DAYS</span>
				</p>
			</div>

			<div class="bg-black/40 border border-[#43D8C9]/20 p-6 rounded-lg relative overflow-hidden">
				<div class="absolute top-0 right-0 w-32 h-32 bg-[#43D8C9]/5 rounded-full blur-xl"></div>
				<p class="text-xs uppercase tracking-widest text-[#43D8C9] mb-2">Total Yield</p>
				<p class="font-orbitron text-4xl font-black flex items-baseline gap-2 text-white">
					{userStats.solEarned} <span class="text-sm font-sans font-normal text-[#43D8C9]">SOL</span>
				</p>
			</div>
		</div>

		<div class="mt-16 p-8 border border-white/10 bg-black/20">
			<h2 class="font-orbitron text-xl mb-6 text-white/80">Recent Ledger Activity</h2>
			<div class="space-y-4 font-mono text-sm opacity-70">
				<div class="flex justify-between border-b border-white/5 pb-2">
					<span>[+50 Pushups] Daily Calibration</span>
					<span class="text-[#43D8C9]">Success</span>
				</div>
				<div class="flex justify-between border-b border-white/5 pb-2">
					<span>[+100 Squats] Leg Day Check-in</span>
					<span class="text-[#43D8C9]">Success</span>
				</div>
				<div class="flex justify-between pb-2">
					<span>[Wallet Init] PDA Generation</span>
					<span class="text-[#95389E]">-0.002 SOL</span>
				</div>
			</div>
		</div>
	</div>
{/if}
