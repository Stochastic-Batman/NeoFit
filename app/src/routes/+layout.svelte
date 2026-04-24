<script lang="ts">
    import '../app.css';
    import { page } from '$app/stores';

    let { children } = $props();

    const envAddress = import.meta.env.VITE_PUBLIC_WALLET_ADDRESS;
    let walletConnected = $state(false);

    let displayAddress = $derived.by(() => {
        if (!walletConnected) return "Connect Wallet";
        if (!envAddress) return "No Address Found";
        return `${envAddress.slice(0, 4)}...${envAddress.slice(-4)}`;
    });

    function toggleWallet() {
        walletConnected = !walletConnected;
    }
</script>



<div class="min-h-screen flex flex-col relative overflow-hidden">
	<div class="absolute top-[-20%] left-[-10%] w-[500px] h-[500px] bg-[#95389E]/10 rounded-full blur-[120px] pointer-events-none"></div>
	<div class="absolute bottom-[-20%] right-[-10%] w-[500px] h-[500px] bg-[#43D8C9]/10 rounded-full blur-[120px] pointer-events-none"></div>

	<nav class="w-full border-b border-white/5 backdrop-blur-md sticky top-0 z-50 bg-[#100303]/80">
		<div class="max-w-6xl mx-auto px-6 h-20 flex items-center justify-between">
			<a href="/" class="flex items-center gap-3 group">
				<div class="relative w-10 h-10 border border-white/10 flex items-center justify-center rotate-45 group-hover:border-[#43D8C9] transition-colors duration-500">
					<div class="absolute w-6 h-6 border-t-2 border-r-2 border-[#95389E] -rotate-45 -translate-x-1 translate-y-1"></div>
					<div class="absolute w-2 h-2 bg-[#43D8C9] rounded-full top-1 right-1"></div>
				</div>
				<span class="font-orbitron font-black text-xl tracking-wider italic">
					NEO<span class="text-[#95389E]">FIT</span>
				</span>
			</a>

			<div class="hidden md:flex items-center gap-8 text-sm uppercase tracking-widest font-semibold">
				<a href="/about" class="{$page.url.pathname === '/about' ? 'text-[#43D8C9]' : 'text-white/60 hover:text-white'} transition-colors">About</a>
				<a href="/challenges" class="{$page.url.pathname === '/challenges' ? 'text-[#43D8C9]' : 'text-white/60 hover:text-white'} transition-colors">Challenges</a>
				{#if walletConnected}
					<a href="/profile" class="{$page.url.pathname === '/profile' ? 'text-[#43D8C9]' : 'text-white/60 hover:text-white'} transition-colors">Profile</a>
	    			{/if}
			</div>


			<button 
			    onclick={toggleWallet}
			    class="font-orbitron text-xs uppercase tracking-widest px-6 py-3 border transition-all duration-300 {walletConnected ? 'border-[#43D8C9] text-[#43D8C9] hover:bg-[#43D8C9]/10' : 'border-[#95389E] text-[#95389E] hover:bg-[#95389E] hover:text-white'}"
			    style="clip-path: polygon(10% 0, 100% 0, 90% 100%, 0 100%);">
			    {displayAddress}
			</button>
		</div>
	</nav>

	<main class="flex-grow w-full max-w-6xl mx-auto px-6 py-12 z-10">
		{@render children()}
	</main>

	<footer class="w-full border-t border-white/5 py-8 z-10">
		<div class="max-w-6xl mx-auto px-6 flex flex-col md:flex-row items-center justify-between text-[10px] uppercase tracking-[0.3em] text-white/30">
			<p>NeoFit v0.0.1</p>
			<p class="mt-4 md:mt-0">Powered by Solana & MoveNet</p>
		</div>
	</footer>
</div>
