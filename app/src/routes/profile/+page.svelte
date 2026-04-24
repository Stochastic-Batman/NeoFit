<script lang="ts">
	// Mock PDA State Data
	let userStats = $state({
		totalReps: 1250,
		streakDays: 12,
		solEarned: 1.45,
		rank: 'Cyber-Athlete'
	});
</script>

<div class="max-w-4xl mx-auto">
	<header class="mb-12 border-b border-white/10 pb-8 flex justify-between items-end">
		<div>
			<h1 class="font-orbitron text-4xl tracking-wider uppercase mb-2 text-[#F7F7F7]">Identity Profile</h1>
			<p class="font-mono text-sm text-[#43D8C9]">Wallet: 5K3x...9aBc</p>
		</div>
		<div class="text-right">
			<p class="text-[10px] uppercase tracking-widest text-white/50 mb-1">Current Classification</p>
			<p class="font-orbitron text-[#95389E] font-bold text-xl">{userStats.rank}</p>
		</div>
	</header>

	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden group">
			<div class="absolute bottom-0 left-0 w-full h-1 bg-[#95389E]"></div>
			<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Lifetime Reps</p>
			<p class="font-orbitron text-5xl font-black">{userStats.totalReps}</p>
		</div>

		<div class="bg-black/40 border border-white/5 p-6 rounded-lg relative overflow-hidden group">
			<div class="absolute bottom-0 left-0 w-full h-1 bg-[#43D8C9]"></div>
			<p class="text-xs uppercase tracking-widest text-white/40 mb-2">Active Streak</p>
			<p class="font-orbitron text-5xl font-black flex items-baseline gap-2">
				{userStats.streakDays} <span class="text-sm font-sans font-normal text-[#43D8C9]">DAYS</span>
			</p>
		</div>

		<div class="bg-black/40 border border-[#43D8C9]/20 p-6 rounded-lg relative overflow-hidden group">
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
