<script lang="ts">
	import { wallet } from '$lib/wallet'

	let selectedDaily = $state<string | null>(null)

	const sponsoredChallenges = [
		{ id: 1, brand: 'Nike', title: 'Weekend Warrior', task: '500 Squats in 48H', pool: '50 SOL', fee: '0.1 SOL', color: '#95389E' }
	]

	const dailyChallenges = [
		{ id: 'd1', title: 'Morning Routine', task: '50 Pushups', reward: '+10 XP' },
		{ id: 'd2', title: 'Leg Day Check-in', task: '100 Squats', reward: '+25 XP' },
		{ id: 'd3', title: 'Endurance Block', task: '200 Jumping Jacks', reward: '+50 XP' }
	]
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

		<section>
			<h2 class="text-xs uppercase tracking-[0.3em] text-[#43D8C9] mb-6 flex items-center gap-2">
				<span class="w-2 h-2 bg-[#43D8C9] rounded-full animate-pulse"></span>
				Sponsored Tournaments
			</h2>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				{#each sponsoredChallenges as challenge}
					<div class="relative p-8 border border-[#95389E]/50 bg-gradient-to-br from-[#95389E]/10 to-transparent group overflow-hidden">
						<div class="absolute top-0 right-0 bg-[#95389E] text-white text-[10px] uppercase font-bold px-3 py-1 font-orbitron">
							Prize Pool: {challenge.pool}
						</div>
						<p class="text-[#43D8C9] text-xs font-bold uppercase tracking-widest mb-1">{challenge.brand} Presents</p>
						<h3 class="font-orbitron text-2xl font-bold mb-4">{challenge.title}</h3>
						<p class="text-white/70 mb-8">{challenge.task}</p>
						<div class="flex items-center justify-between mt-auto">
							<span class="text-sm font-mono opacity-60">Entry: {challenge.fee}</span>
							<button class="border border-[#95389E] px-6 py-2 font-orbitron text-xs uppercase tracking-widest hover:bg-[#95389E] transition-colors">
								Join Pool
							</button>
						</div>
					</div>
				{/each}
			</div>
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
