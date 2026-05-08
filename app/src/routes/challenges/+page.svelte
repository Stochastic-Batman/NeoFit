<script lang="ts">
	import { wallet } from '$lib/wallet'
	import { fetchAllChallenges, joinChallenge, claimReward, fetchEnrollment } from '$lib/program'
	import { PublicKey } from '@solana/web3.js'

	let selectedDaily = $state<string | null>(null)
	let challenges = $state<any[]>([])
	let enrollments = $state<Record<string, any>>({}) // keyed by challenge pubkey base58
	let loadingChallenges = $state(false)
	let challengeErr = $state('')
	let actionLoading = $state<string | null>(null) // challenge pubkey currently being acted on
	let actionErr = $state('')
	let actionMsg = $state('')

	const dailyChallenges = [
		{ id: 'd1', title: 'Morning Routine', task: '50 Pushups', reward: '+10 XP' },
		{ id: 'd2', title: 'Leg Day Check-in', task: '100 Squats', reward: '+25 XP' },
		{ id: 'd3', title: 'Endurance Block', task: '200 Jumping Jacks', reward: '+50 XP' }
	]

	$effect(() => {
		if ($wallet.connected) {
			loadChallenges()
		} else {
			challenges = []
			enrollments = {}
		}
	})

	async function loadChallenges() {
		loadingChallenges = true
		challengeErr = ''
		try {
			const result = await fetchAllChallenges()
			challenges = result
			// Check enrollment for each challenge
			if ($wallet.publicKey) {
				const enrollMap: Record<string, any> = {}
				for (const c of result) {
					try {
						const enrollment = await fetchEnrollment(c.publicKey, $wallet.publicKey)
						if (enrollment) enrollMap[c.publicKey.toBase58()] = enrollment
					} catch {}
				}
				enrollments = enrollMap
			}
		} catch (e: any) {
			challengeErr = e.message
		} finally {
			loadingChallenges = false
		}
	}

	async function handleJoin(challengeKey: PublicKey) {
		actionLoading = challengeKey.toBase58()
		actionErr = ''
		actionMsg = ''
		try {
			const sig = await joinChallenge(challengeKey)
			actionMsg = `Joined! tx: ${sig.slice(0, 8)}…`
			// Reload to update enrollment status
			await loadChallenges()
		} catch (e: any) {
			actionErr = e.message
		} finally {
			actionLoading = null
		}
	}

	async function handleClaim(challengeKey: PublicKey) {
		actionLoading = challengeKey.toBase58()
		actionErr = ''
		actionMsg = ''
		try {
			const sig = await claimReward(challengeKey)
			actionMsg = `Claimed! tx: ${sig.slice(0, 8)}…`
			await loadChallenges()
		} catch (e: any) {
			actionErr = e.message
		} finally {
			actionLoading = null
		}
	}

	function formatLamports(lamports: number | bigint): string {
		return (Number(lamports) / 1_000_000_000).toFixed(2)
	}

	function formatDeadline(ts: number | bigint): string {
		const date = new Date(Number(ts) * 1000)
		return date.toLocaleString()
	}

	function isExpired(ts: number | bigint): boolean {
		return Date.now() > Number(ts) * 1000
	}
</script>

{#if !$wallet.connected}
	<div class="flex flex-col items-center justify-center min-h-[50vh] text-center gap-6">
		<div class="w-16 h-16 border border-white/10 flex items-center justify-center rotate-45">
			<div class="w-6 h-6 border-t-2 border-r-2 border-[#95389E] -rotate-45"></div>
		</div>
		<h2 class="font-orbitron text-2xl uppercase tracking-wider">Wallet Required</h2>
		<p class="text-white/40 max-w-sm">Connect your wallet to access challenges and earn rewards on-chain.</p>
		<button
			onclick={() => wallet.connect()}
			class="border border-[#95389E] text-[#95389E] font-orbitron text-xs uppercase tracking-widest px-8 py-3 hover:bg-[#95389E] hover:text-white transition-all"
			style="clip-path: polygon(5% 0, 100% 0, 95% 100%, 0 100%);">
			Connect Wallet
		</button>
	</div>
{:else}
	<div class="space-y-16">
		<header>
			<h1 class="font-orbitron text-4xl text-[#95389E] tracking-wider uppercase mb-2">Active Protocols</h1>
			<p class="text-white/50 text-sm">Select a challenge to calibrate the vision network.</p>
		</header>

		{#if actionMsg}
			<p class="text-[#43D8C9] text-xs font-mono">{actionMsg}</p>
		{/if}
		{#if actionErr}
			<p class="text-red-400 text-xs font-mono">{actionErr}</p>
		{/if}

		<section>
			<h2 class="text-xs uppercase tracking-[0.3em] text-[#43D8C9] mb-6 flex items-center gap-2">
				<span class="w-2 h-2 bg-[#43D8C9] rounded-full animate-pulse"></span>
				On-Chain Challenges
			</h2>

			{#if loadingChallenges}
				<p class="text-white/40 font-mono text-sm animate-pulse">Loading challenges...</p>
			{:else if challengeErr}
				<p class="text-red-400 text-xs font-mono">{challengeErr}</p>
			{:else if challenges.length === 0}
				<div class="border border-white/10 p-8 text-center">
					<p class="text-white/40 font-mono text-sm">No challenges on-chain yet.</p>
					<p class="text-white/20 text-xs mt-2">Run the seed script to create one.</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					{#each challenges as { publicKey, account }}
						{@const keyStr = publicKey.toBase58()}
						{@const enrollment = enrollments[keyStr]}
						{@const expired = isExpired(account.deadlineTs)}
						<div class="relative p-8 border border-[#95389E]/50 bg-gradient-to-br from-[#95389E]/10 to-transparent group overflow-hidden">
							<div class="absolute top-0 right-0 bg-[#95389E] text-white text-[10px] uppercase font-bold px-3 py-1 font-orbitron">
								Pool: {formatLamports(account.poolLamports)} SOL
							</div>
							<h3 class="font-orbitron text-2xl font-bold mb-2">{account.title}</h3>
							<p class="text-white/50 text-xs font-mono mb-4">
								Deadline: {formatDeadline(account.deadlineTs)}
								{#if expired}<span class="text-red-400 ml-2">(Expired)</span>{/if}
							</p>
							<div class="mb-4 space-y-1">
								{#each account.requirements as req, i}
									{@const logged = enrollment?.repsLogged?.[i]?.count ?? 0}
									<div class="flex items-center gap-2">
										<p class="text-white/70 text-sm flex-1">Exercise #{req.exerciseId}: {req.repTarget} reps</p>
										{#if enrollment}
											<span class="text-[#43D8C9] text-xs font-mono">{logged}/{req.repTarget}</span>
										{/if}
									</div>
								{/each}
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm font-mono opacity-60">Entry: {formatLamports(account.entryFeeLamports)} SOL</span>

								{#if enrollment}
									{#if enrollment.rewardClaimed}
										<span class="text-[#43D8C9] font-orbitron text-xs uppercase">Claimed ✓</span>
									{:else if enrollment.completed && expired}
										<button
											onclick={() => handleClaim(publicKey)}
											disabled={actionLoading === keyStr}
											class="border border-[#43D8C9] text-[#43D8C9] font-orbitron text-[10px] uppercase tracking-widest px-4 py-2 hover:bg-[#43D8C9]/10 transition-colors disabled:opacity-50">
											{actionLoading === keyStr ? '…' : 'Claim Reward'}
										</button>
									{:else}
										<span class="text-[#95389E] font-orbitron text-xs uppercase">Enrolled</span>
									{/if}
								{:else if !expired && account.isActive}
									<button
										onclick={() => handleJoin(publicKey)}
										disabled={actionLoading === keyStr}
										class="border border-[#95389E] text-[#95389E] font-orbitron text-[10px] uppercase tracking-widest px-4 py-2 hover:bg-[#95389E]/10 transition-colors disabled:opacity-50">
										{actionLoading === keyStr ? '…' : 'Join Pool'}
									</button>
								{:else}
									<span class="text-xs font-mono text-white/30">{account.isActive ? 'Active' : 'Inactive'}</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>

		<section>
			<h2 class="text-xs uppercase tracking-[0.3em] text-white/50 mb-6">Daily Calibration Routines</h2>

			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				{#each dailyChallenges as daily}
					<button
						type="button"
						onclick={() => selectedDaily = daily.id}
						class="w-full text-left p-6 border transition-all cursor-pointer
							{selectedDaily === daily.id
								? 'border-[#43D8C9] bg-[#43D8C9]/10'
								: 'border-white/10 hover:border-white/30 bg-black/50'}">
						<div class="flex justify-between items-start mb-4">
							<h3 class="font-orbitron font-bold text-lg text-white">{daily.title}</h3>
							{#if selectedDaily === daily.id}
								<div class="w-3 h-3 bg-[#43D8C9] rounded-full"></div>
							{/if}
						</div>
						<p class="text-sm text-white/60 mb-4">{daily.task}</p>
						<p class="text-xs font-mono text-[#95389E]">{daily.reward}</p>
					</button>
				{/each}
			</div>

			{#if selectedDaily}
				<div class="mt-8 flex justify-end">
					<a href="/workout" class="bg-[#43D8C9] text-[#100303] font-orbitron font-bold px-8 py-3 uppercase tracking-widest hover:shadow-[0_0_20px_rgba(67,216,201,0.4)] transition-all">
						Begin Daily
					</a>
				</div>
			{/if}
		</section>
	</div>
{/if}
