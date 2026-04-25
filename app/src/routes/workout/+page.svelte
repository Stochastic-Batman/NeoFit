<script lang="ts">
	import { onMount, onDestroy } from 'svelte'
	import { workouts } from '$lib/workouts'
	import type { St } from '$lib/workouts'

	// DOM refs
	let vid = $state<HTMLVideoElement | null>(null)
	let cvs = $state<HTMLCanvasElement | null>(null)

	// UI state
	let selId = $state(workouts[0].id)
	let open = $state(false)
	let count = $state(0)
	let phase = $state('—')
	let camOk = $state(false)
	let netOk = $state(false)
	let err = $state('')
	let wSt = $state<St>({})

	const cur = $derived(workouts.find((w) => w.id === selId)!)

	// Skeleton edges (MoveNet 17-point)
	const EDGES: [string, string][] = [
		['nose', 'left_eye'], ['nose', 'right_eye'],
		['left_eye', 'left_ear'], ['right_eye', 'right_ear'],
		['left_shoulder', 'right_shoulder'],
		['left_shoulder', 'left_elbow'], ['left_elbow', 'left_wrist'],
		['right_shoulder', 'right_elbow'], ['right_elbow', 'right_wrist'],
		['left_shoulder', 'left_hip'], ['right_shoulder', 'right_hip'],
		['left_hip', 'right_hip'],
		['left_hip', 'left_knee'], ['left_knee', 'left_ankle'],
		['right_hip', 'right_knee'], ['right_knee', 'right_ankle']
	]

	// Keypoints worth highlighting in violet (exercise-relevant joints)
	const ACCENT = new Set([
		'left_shoulder', 'right_shoulder',
		'left_elbow', 'right_elbow',
		'left_wrist', 'right_wrist',
		'left_hip', 'right_hip',
		'left_knee', 'right_knee',
		'left_ankle', 'right_ankle',
		'nose', 'left_ear', 'right_ear',  // I think these will be useful for exercises that need head movement
	])

	// Draw callback (called by ml5 per frame)
	function onPoses(poses: any[]) {
		if (!cvs || !vid) return
		
		const ctx = cvs.getContext('2d')!
		const W = vid.videoWidth, H = vid.videoHeight
		if (!W || !H) return

		if (cvs.width !== W) { 
		    cvs.width = W; 
		    cvs.height = H 
		}

		// Draw mirrored video frame
		ctx.save()
		ctx.scale(-1, 1)
		ctx.drawImage(vid, -W, 0, W, H)
		ctx.restore()

		if (!poses.length) return
		const pose = poses[0]

		// Index keypoints by name
		const kps: Record<string, any> = {}
		for (const k of pose.keypoints) kps[k.name] = k

		// Draw edges — mirror x by drawing at W - x
		ctx.lineWidth = 2
		ctx.strokeStyle = '#43D8C9'
		for (const [a, b] of EDGES) {
			const ka = kps[a], kb = kps[b]
			if (ka?.confidence > 0.3 && kb?.confidence > 0.3) {
				ctx.beginPath()
				ctx.moveTo(W - ka.x, ka.y)
				ctx.lineTo(W - kb.x, kb.y)
				ctx.stroke()
			}
		}

		// Draw joint dots
		for (const k of pose.keypoints) {
			if (k.confidence <= 0.3) continue
			ctx.beginPath()
			ctx.arc(W - k.x, k.y, ACCENT.has(k.name) ? 6 : 4, 0, 2 * Math.PI)
			ctx.fillStyle = ACCENT.has(k.name) ? '#95389E' : '#43D8C9'
			ctx.fill()
		}

		// Run workout logic (original unflipped coords — angles are flip-invariant)
		const res = cur.detect(pose, wSt)
		count = res.count
		phase = res.phase
		wSt = res.st
	}

	function reset() {
		wSt = cur.init()
		count = 0
		phase = '—'
	}

	function pick(id: string) {
		selId = id
		open = false
		reset()
	}

	// ml5 handle
	let bp: any = null

	onMount(async () => {
		// Camera
		try {
			const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' } })
			vid!.srcObject = stream
			await vid!.play()
			camOk = true
		} catch (e: any) {
			err = `Camera error: ${e.message}`
			return
		}

		// MoveNet
		try {
			const { default: ml5 } = await import('ml5')
			await new Promise<void>((res) => {
				bp = ml5.bodyPose('MoveNet', {}, res)
			})
			netOk = true
			wSt = cur.init()
			bp.detectStart(vid!, onPoses)
		} catch (e: any) {
			err = `MoveNet error: ${e.message}`
		}
	})

	onDestroy(() => {
		bp?.detectStop?.()
		;(vid?.srcObject as MediaStream | null)?.getTracks().forEach((t) => t.stop())
	})
</script>

<div class="space-y-8">
	<!-- Header -->
	<header>
		<h1 class="font-orbitron text-4xl tracking-wider uppercase mb-2">
			<span class="text-[#95389E]">Motion</span> Capture
		</h1>
		<p class="text-white/40 text-sm tracking-wide">
			Select an exercise, position yourself, and let the network verify your reps.
		</p>
	</header>

	<!-- Exercise picker -->
	<div class="relative z-30 w-full max-w-xs">
		<button
			type="button"
			onclick={() => (open = !open)}
			class="w-full flex items-center justify-between border px-4 py-3 font-orbitron text-xs uppercase tracking-widest transition-all
				{open ? 'border-[#43D8C9] text-[#43D8C9]' : 'border-white/10 text-white/80 hover:border-white/30'}"
		>
			{cur.name}
			<span class="ml-4 text-[8px]">{open ? '▲' : '▼'}</span>
		</button>

		{#if open}
			<div class="absolute top-full left-0 right-0 border border-[#43D8C9]/40 bg-[#100303] divide-y divide-white/5 shadow-[0_8px_32px_rgba(67,216,201,0.1)]">
				{#each workouts as w}
					<button
						type="button"
						onclick={() => pick(w.id)}
						class="w-full text-left px-4 py-3 font-orbitron text-xs uppercase tracking-widest transition-colors
							{selId === w.id
								? 'text-[#43D8C9] bg-[#43D8C9]/10'
								: 'text-white/50 hover:text-white hover:bg-white/5'}"
					>
						{w.name}
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Error state -->
	{#if err}
		<div class="border border-red-500/30 bg-red-500/5 p-4 font-mono text-red-400 text-sm">
			{err}
		</div>
	{:else}
		<div class="grid grid-cols-1 lg:grid-cols-[1fr_260px] gap-6">

			<!-- Camera canvas -->
			<div class="relative bg-black border border-white/10 overflow-hidden" style="aspect-ratio: 4/3;">
				<!-- Loading overlays -->
				{#if !camOk}
					<div class="absolute inset-0 flex flex-col items-center justify-center gap-3">
						<div class="w-8 h-8 border-2 border-[#43D8C9] border-t-transparent rounded-full animate-spin"></div>
						<p class="font-orbitron text-xs uppercase tracking-widest text-white/30 animate-pulse">
							Requesting camera…
						</p>
					</div>
				{:else if !netOk}
					<div class="absolute inset-0 flex flex-col items-center justify-center gap-3 z-10 bg-black/60">
						<div class="w-8 h-8 border-2 border-[#95389E] border-t-transparent rounded-full animate-spin"></div>
						<p class="font-orbitron text-xs uppercase tracking-widest text-white/50 animate-pulse">
							Loading MoveNet…
						</p>
					</div>
				{/if}

				<!-- Hidden video source -->
				<!-- svelte-ignore a11y_media_has_caption -->
				<video bind:this={vid} class="hidden" playsinline muted></video>

				<!-- Canvas — draws mirrored video + skeleton -->
				<canvas
					bind:this={cvs}
					class="absolute inset-0 w-full h-full"
				></canvas>

				<!-- Corner decorations (purely visual) -->
				<div class="absolute top-0 left-0 w-6 h-6 border-t-2 border-l-2 border-[#43D8C9] pointer-events-none"></div>
				<div class="absolute top-0 right-0 w-6 h-6 border-t-2 border-r-2 border-[#43D8C9] pointer-events-none"></div>
				<div class="absolute bottom-0 left-0 w-6 h-6 border-b-2 border-l-2 border-[#43D8C9] pointer-events-none"></div>
				<div class="absolute bottom-0 right-0 w-6 h-6 border-b-2 border-r-2 border-[#43D8C9] pointer-events-none"></div>

				<!-- Live indicator -->
				{#if netOk}
					<div class="absolute top-4 right-10 flex items-center gap-2 pointer-events-none">
						<span class="w-2 h-2 bg-[#43D8C9] rounded-full animate-pulse"></span>
						<span class="font-orbitron text-[9px] uppercase tracking-widest text-[#43D8C9]">Live</span>
					</div>
				{/if}
			</div>

			<!-- Stats panel -->
			<div class="flex flex-col gap-4">
				<!-- Rep count -->
				<div class="border border-white/10 bg-black/40 p-6 relative overflow-hidden">
					<div class="absolute bottom-0 left-0 w-full h-0.5 bg-[#95389E]"></div>
					<p class="text-[10px] uppercase tracking-[0.3em] text-white/30 mb-2">Reps</p>
					<p class="font-orbitron text-7xl font-black leading-none tabular-nums">{count}</p>
				</div>

				<!-- Phase -->
				<div class="border border-white/10 bg-black/40 p-6 relative overflow-hidden">
					<div class="absolute bottom-0 left-0 w-full h-0.5 bg-[#43D8C9]"></div>
					<p class="text-[10px] uppercase tracking-[0.3em] text-white/30 mb-2">Phase</p>
					<p
						class="font-orbitron text-2xl font-bold uppercase tracking-widest transition-colors"
						class:text-[#43D8C9]={phase === 'up' || phase === 'armsUp'}
						class:text-[#95389E]={phase === 'down'}
						class:text-white={phase === '—'}
					>
						{phase}
					</p>
				</div>

				<!-- Tip -->
				<div class="border border-[#43D8C9]/20 bg-[#43D8C9]/5 p-4">
					<p class="text-[10px] uppercase tracking-widest text-[#43D8C9] mb-2">Positioning</p>
					<p class="text-sm text-white/60 leading-relaxed">{cur.cue}</p>
				</div>

				<!-- Reset -->
				<button
					type="button"
					onclick={reset}
					class="border border-white/10 py-3 font-orbitron text-xs uppercase tracking-widest text-white/40 hover:border-[#95389E] hover:text-[#95389E] transition-all"
				>
					Reset Counter
				</button>
			</div>
		</div>
	{/if}
</div>
